use super::{PickSelection, PickingPluginsSettings, Settings};
use crate::events::{console::*, *};
use crate::print_analyzer::Parsed;
use crate::GCode;
use bevy::input::mouse::MouseMotion;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiContexts};
use bevy_mod_picking::prelude::*;
use egui::Pos2;

#[derive(PartialEq, Clone, Copy)]
pub enum Choice {
    Vertex,
    Shape,
    Layer,
}

#[derive(PartialEq)]
enum Cursor {
    Pointer,
    Brush,
    Eraser,
}

#[derive(Resource)]
pub struct UiResource {
    pub display_z_max: (f32, f32),
    pub display_z_min: f32,
    pub vertex_counter: u32,
    pub selection_enum: Choice,
    pub subdivide_slider: u32,
    pub gcode_emit: String,
    pub vis_select: VisibilitySelector,
    cursor_enum: Cursor,
}

impl Default for UiResource {
    fn default() -> Self {
        UiResource {
            display_z_max: (0.0, 0.0),
            display_z_min: 0.0,
            vertex_counter: 0,
            selection_enum: Choice::Vertex,
            subdivide_slider: 1,
            gcode_emit: String::new(),
            vis_select: VisibilitySelector::default(),
            cursor_enum: Cursor::Pointer,
        }
    }
}

pub struct VisibilitySelector {
    pub extrusion: bool,
    pub wipe: bool,
    pub retraction: bool,
    pub deretraction: bool,
    pub travel: bool,
    pub preprint: bool,
}
impl Default for VisibilitySelector {
    fn default() -> Self {
        VisibilitySelector {
            extrusion: true,
            wipe: false,
            retraction: false,
            deretraction: false,
            travel: false,
            preprint: false,
        }
    }
}

pub fn ui_setup(gcode: Res<GCode>, mut ui_res: ResMut<UiResource>) {
    for (_, v) in gcode.0.vertices.iter() {
        ui_res.display_z_max.1 = ui_res.display_z_max.1.max(v.to.z);
        ui_res.vertex_counter = ui_res.vertex_counter.max(v.count);
    }
    ui_res.display_z_max.0 = ui_res.display_z_max.1;
}
pub fn toolbar(mut contexts: EguiContexts, mut system_writer: EventWriter<SystemEvent>) {
    let ctx = contexts.ctx_mut();
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Export GCode (Ctrl+S)").clicked() {
                    system_writer.send(SystemEvent::SaveAs);
                }
            });
            ui.menu_button("Transform", |ui| if ui.button("Rotate").clicked() {})
        })
    });
}

#[derive(Resource)]
pub struct RightClick(Pos2);

pub fn right_click(
    mut commands: Commands,
    motion_reader: EventReader<MouseMotion>,
    mut egui_context: Query<&mut EguiContext>,
    click: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    // FIXME: want to know if mouse moved at all
    let drag = !motion_reader.is_empty();
    if let Some(Vec2 { x, y }) = window.get_single().unwrap().cursor_position() {
        if let Ok(mut context) = egui_context.get_single_mut() {
            let context = context.get_mut();
            let pos = Pos2 { x, y };
            if click.just_released(MouseButton::Right) && !drag {
                commands.insert_resource(RightClick(pos));
            }
            if click.just_pressed(MouseButton::Left) && !context.wants_pointer_input() {
                commands.remove_resource::<RightClick>();
            }
        }
    }
}

pub fn right_click_menu(mut contexts: EguiContexts, pos: Res<RightClick>) {
    egui::Window::new("right click")
        .title_bar(false)
        .resizable(false)
        .fixed_pos(pos.0)
        .show(contexts.ctx_mut(), |ui| if ui.button("asdf").clicked() {});
}

#[derive(Default, Resource)]
pub struct ConsoleActive(bool);

pub fn console(
    mut contexts: EguiContexts,
    mut console: ResMut<Console>,
    mut console_active: ResMut<ConsoleActive>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let height = window.height() / 5.0;
    let width = contexts.ctx_mut().available_rect().width();
    egui::TopBottomPanel::bottom("console")
        //.min_height(height)
        .resizable(true)
        .show_separator_line(true)
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                // .min_scrolled_height(height)
                .max_height(height)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::bottom_up(egui::Align::LEFT).with_cross_justify(true),
                        |ui| {
                            ui.label(egui::RichText::new(&console.output).small().weak());
                        },
                    );
                });
            // update resource to reflect console focused
            console_active.0 = {
                let input = egui::TextEdit::singleline(&mut console.input)
                    .desired_width(width);
                ui.add(input).has_focus()
            };
        });
}

pub fn sidebar(
    mut contexts: EguiContexts,
    mut system_writer: EventWriter<SystemEvent>,
    vertex: Res<VertexCounter>,
    mut ui_res: ResMut<UiResource>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = primary_window.single();
    let height = window.height();
    let spacing = height / 50.0;
    let max = vertex.max;
    egui::SidePanel::new(egui::panel::Side::Left, "panel")
        //.exact_width(panel_width)
        .resizable(true)
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label("world");
                ui.add_space(spacing);
                ui.add(egui::Slider::new(&mut ui_res.vertex_counter, 0..=max));
                ui.add_space(spacing);
                let mx = ui_res.display_z_max.1;
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut ui_res.display_z_max.0, 0.0..=mx)
                            .vertical()
                            .step_by(0.1),
                    );
                    ui.add(
                        egui::Slider::new(&mut ui_res.display_z_min, mx..=0.0)
                            .vertical()
                            .step_by(0.1),
                    );
                });
                let steps = [
                    (100, "<<<"),
                    (10, "<<"),
                    (1, "<"),
                    (1, ">"),
                    (10, ">>"),
                    (100, ">>>"),
                ];
                let mut i = 0;
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    for (num, str) in steps {
                        let neg = i < steps.len() / 2;
                        if ui.button(str).clicked() {
                            if neg {
                                ui_res.vertex_counter -= num;
                            } else {
                                ui_res.vertex_counter += num;
                            }
                        }
                        i += 1;
                    }
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut ui_res.cursor_enum, Cursor::Pointer, "Pointer");
                    ui.radio_value(&mut ui_res.cursor_enum, Cursor::Brush, "Brush");
                    ui.radio_value(&mut ui_res.cursor_enum, Cursor::Eraser, "Eraser");
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    ui.radio_value(&mut ui_res.selection_enum, Choice::Vertex, "Vertex");
                    ui.radio_value(&mut ui_res.selection_enum, Choice::Shape, "Shape");
                    ui.radio_value(&mut ui_res.selection_enum, Choice::Layer, "Layer");
                });

                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    let _ = ui.checkbox(&mut ui_res.vis_select.extrusion, "extrusion");
                    let _ = ui.checkbox(&mut ui_res.vis_select.travel, "travel");
                    let _ = ui.checkbox(&mut ui_res.vis_select.retraction, "retraction");
                    let _ = ui.checkbox(&mut ui_res.vis_select.wipe, "wipe");
                    let _ = ui.checkbox(&mut ui_res.vis_select.deretraction, "deretraction");
                    let _ = ui.checkbox(&mut ui_res.vis_select.preprint, "preprint");
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    if ui.button("refresh").clicked() {
                        system_writer.send(SystemEvent::ForceRefresh);
                    }
                });
                ui.add_space(spacing);
                ui.text_edit_multiline(&mut ui_res.gcode_emit)
                    .on_hover_text("enter custom gcode");
                ui.add_space(spacing)
            });
        });
}

#[derive(Resource)]
pub struct VertexCounter {
    max: u32,
}

impl VertexCounter {
    pub fn build(gcode: &Parsed) -> VertexCounter {
        VertexCounter {
            max: gcode.vertices.keys().len() as u32,
        }
    }
}

pub fn key_system(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    settings: Res<Settings>,
    mut system_writer: EventWriter<SystemEvent>,
    console_active: Res<ConsoleActive>,
    mut ui_writer: EventWriter<UiEvent>,
    mut console: ResMut<Console>,
) {
    if console_active.0 {
        if keys.just_pressed(KeyCode::Enter) {
            
            println!("{}",console.input);
            let output: String = console.input.drain(..).collect();
            println!("{}",output);
            console.output.push_str(&output);
            ui_writer.send(UiEvent::ConsoleEnter(output));

        }
        return;
    }
    if keys.pressed(KeyCode::ArrowLeft) {
        ui_writer.send(UiEvent::MoveDisplay(false, false, 1.0));
    } else if keys.pressed(KeyCode::ArrowRight) {
        ui_writer.send(UiEvent::MoveDisplay(true, false, 1.0));
    } else if keys.pressed(KeyCode::ArrowUp) {
        ui_writer.send(UiEvent::MoveDisplay(true, true, 0.2));
    } else if keys.pressed(KeyCode::ArrowDown) {
        ui_writer.send(UiEvent::MoveDisplay(false, true, -0.2));
    }
    // check for ctrl press, and then check if shift also held
    // i should also check here if i should be entering into the console
    else if keys.any_pressed([KeyCode::ControlRight, KeyCode::ControlLeft]) {
        // if not shift
        if !keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            if keys.just_pressed(KeyCode::KeyZ) {
                ui_writer.send(UiEvent::Undo);
            } else if keys.just_pressed(KeyCode::KeyR) {
                system_writer.send(SystemEvent::ForceRefresh);
            } else if keys.just_pressed(KeyCode::KeyS) {
                system_writer.send(SystemEvent::SaveAs);
            } else if keys.just_pressed(KeyCode::KeyA) {
                ui_writer.send(UiEvent::SelectAll);
            }
        } else if keys.just_pressed(KeyCode::KeyZ) {
            ui_writer.send(UiEvent::Redo);
        }
    } else if keys.just_pressed(settings.hole_delete_button) {
        ui_writer.send(UiEvent::HoleDelete);
    } else if keys.just_pressed(settings.merge_delete_button) {
        ui_writer.send(UiEvent::MergeDelete);
    }
    // clear key presses after read
    keys.clear();
}

pub fn select_erase_brush(
    mut shift: ResMut<ButtonInput<KeyCode>>,
    mut hover_reader: EventReader<Pointer<Over>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut s_query: Query<(Entity, &mut PickSelection)>,
    ui_res: Res<UiResource>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui_writer: EventWriter<UiEvent>,
) {
    if let Ok(mut window) = window.get_single_mut() {
        if ui_res.cursor_enum == Cursor::Pointer {
            window.cursor.icon = CursorIcon::Pointer;
            return;
        }
        window.cursor.icon = CursorIcon::Crosshair;
    } else {
        return;
    }
    shift.press(KeyCode::ShiftLeft);
    ui_writer.send(UiEvent::SetPanOrbit(false));
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    for hover in hover_reader.read() {
        if let Ok((_, mut selection)) = s_query.get_mut(hover.target) {
            selection.is_selected = ui_res.cursor_enum == Cursor::Brush;
        }
    }
}

pub fn capture_mouse(
    mut pick_settings: ResMut<PickingPluginsSettings>,
    mut egui_context: Query<&mut EguiContext>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut pan_orbit: EventWriter<UiEvent>,
) {
    if let Ok(mut context) = egui_context.get_single_mut() {
        let context = context.get_mut();
        if context.is_using_pointer() || context.wants_pointer_input() {
            pick_settings.is_enabled = false;
            pan_orbit.send(UiEvent::SetPanOrbit(false));
        } else if !mouse.any_pressed([MouseButton::Left, MouseButton::Right]) {
            pick_settings.is_enabled = true;
            pan_orbit.send(UiEvent::SetPanOrbit(true));
        }
    }
}
