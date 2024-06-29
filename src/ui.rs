use super::diff::{SelectionLog, SetSelections};
use super::{
    EguiContext, HoleDelete, MergeDelete, PickSelection, PickingPluginsSettings, Settings,
    SubdivideSelection,
};
use crate::print_analyzer::Parsed;
use crate::{ForceRefresh, GCode, Tag};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use bevy_mod_picking::{prelude::*, selection::SelectionPluginSettings};
use egui::Pos2;
use std::collections::HashSet;

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
    subdivide_slider: u32,
    translation_input: String,
    pub gcode_emit: String,
    pub vis_select: VisibilitySelector,
    pub rotate_x: f32,
    pub rotate_y: f32,
    pub rotate_z: f32,
    pub scale: f32,
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
            translation_input: String::new(),
            gcode_emit: String::new(),
            vis_select: VisibilitySelector::default(),
            rotate_x: 0.0,
            rotate_y: 0.0,
            rotate_z: 0.0,
            scale: 1.0,
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
pub fn toolbar(mut contexts: EguiContexts) {
    egui::TopBottomPanel::top("toolbar").show(contexts.ctx_mut(), |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    // …
                }
                else if ui.button("Export GCode").clicked() {}
            });
            ui.menu_button("Transform", |ui| {
                if ui.button("Rotate").clicked() {}
            })
        })
    });
}

#[derive(Default, Resource)]
pub struct RightClick;

pub fn right_click(mut commands: Commands, click: Res<ButtonInput<MouseButton>>) {
    if click.just_pressed(MouseButton::Right) {
        commands.init_resource::<RightClick>();
    }
    if click.just_pressed(MouseButton::Left) {
        commands.remove_resource::<RightClick>();
    }
}

pub fn right_click_menu(mut contexts: EguiContexts, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.get_single().unwrap();
    if let Some(Vec2{x,y}) = window.cursor_position(){
        let min = Pos2 {x, y};
        let max = Pos2 {x, y};
        let rect = egui::Rect {min, max};
        egui::popup::show_tooltip_for(contexts.ctx_mut(), "right click popup".into(), &rect, |ui| {
            egui::ComboBox::new("right click box", "context menu").show_ui(ui, |ui| {
            });
        });
    }
    
}

pub fn ui_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    vertex: Res<VertexCounter>,
    mut ui_res: ResMut<UiResource>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut gcode: ResMut<GCode>,
    s_query: Query<(&mut PickSelection, &Tag)>,
) {
    let Ok(window) = window.get_single() else {
        panic!();
    };
    let panel_width = window.width() / 6.0;
    let height = window.height();
    let spacing = height / 50.0;
    let max = vertex.max;
    let mut selection = HashSet::new();
    for (pick, id) in s_query.iter() {
        if pick.is_selected {
            selection.insert(id.id);
        }
    }
    egui::SidePanel::new(egui::panel::Side::Left, "panel")
        .exact_width(panel_width)
        .resizable(false)
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
                    if ui.button("Merge Delete").clicked() {
                        commands.init_resource::<MergeDelete>();
                    } else if ui.button("Hole Delete").clicked() {
                        commands.init_resource::<HoleDelete>();
                    }
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    let _response = ui.add(egui::Slider::new(&mut ui_res.subdivide_slider, 1..=10));
                    if ui.button("Subdivide to max distance").clicked() {
                        commands.insert_resource(SubdivideSelection(ui_res.subdivide_slider));
                    }
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
                    let _response = ui.text_edit_singleline(&mut ui_res.translation_input);

                    let enu = ui_res.selection_enum;
                    if ui.button("Translate").clicked() && !selection.is_empty() {
                        if ui_res.translation_input.is_empty() {
                            return;
                        }
                        let mut params = ui_res.translation_input.split_whitespace();
                        let x = params.next().unwrap().parse::<f32>().unwrap();
                        let y = params.next().unwrap().parse::<f32>().unwrap();
                        let z = params.next().unwrap().parse::<f32>().unwrap();
                        match enu {
                            Choice::Vertex => {
                                for selection in &selection {
                                    gcode.0.translate(selection, x, y, z);
                                }
                            }
                            Choice::Shape => {
                                let mut shapes = HashSet::new();
                                for selection in &selection {
                                    let shape = gcode.0.get_shape(selection);
                                    shapes.extend(&shape);
                                }
                                for vertex in shapes.iter() {
                                    gcode.0.translate(vertex, x, y, z);
                                }
                            }
                            Choice::Layer => {
                                let mut layers = HashSet::new();
                                for selection in &selection {
                                    let layer = gcode.0.get_same_z(selection);
                                    layers.extend(&layer);
                                }
                                for vertex in layers.iter() {
                                    gcode.0.translate(vertex, x, y, z);
                                }
                            }
                        }
                        commands.init_resource::<ForceRefresh>();
                    }
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    if ui.button("refresh").clicked() {
                        commands.insert_resource(ForceRefresh);
                    }
                });
                ui.add_space(spacing);
                ui.text_edit_multiline(&mut ui_res.gcode_emit)
                    .on_hover_text("enter custom gcode");
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut ui_res.rotate_x, -180.0..=180.0).vertical());
                    ui.add(egui::Slider::new(&mut ui_res.rotate_y, -180.0..=180.0).vertical());
                    ui.add(egui::Slider::new(&mut ui_res.rotate_z, -180.0..=180.0).vertical());
                    if ui.button("Rotate").clicked() {
                        let origin = gcode.0.get_centroid(&selection);
                        for vertex in &selection {
                            gcode.0.rotate(
                                vertex,
                                origin,
                                ui_res.rotate_x,
                                ui_res.rotate_y,
                                ui_res.rotate_z,
                            );
                        }
                        commands.init_resource::<ForceRefresh>();
                    }
                });
                ui.add_space(spacing);
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut ui_res.scale, 0.1..=10.0));
                    if ui.button("Scale").clicked() {
                        let origin = gcode.0.get_centroid(&selection);
                        for vertex in &selection {
                            gcode.0.scale(vertex, origin, ui_res.scale);
                        }
                        commands.init_resource::<ForceRefresh>();
                    }
                });
                if ui.button("Save").clicked() {
                    let _ = gcode.0.write_to_file("./test_output.gcode");
                }
            })
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
    mut commands: Commands,
    mut ui_res: ResMut<UiResource>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut log: ResMut<SelectionLog>,
    settings: Res<Settings>,
) {
    if keys.pressed(KeyCode::ArrowLeft) {
        if ui_res.vertex_counter == 0 {
            return;
        }
        ui_res.vertex_counter -= 1;
    } else if keys.pressed(KeyCode::ArrowRight) {
        ui_res.vertex_counter += 1;
    } else if keys.pressed(KeyCode::ArrowUp) {
        ui_res.display_z_max.0 += 0.2;
    } else if keys.pressed(KeyCode::ArrowDown) {
        ui_res.display_z_max.0 -= 0.2;
    } else if keys.any_pressed([KeyCode::ControlRight, KeyCode::ControlLeft])
        && keys.just_pressed(KeyCode::KeyZ)
        && !keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        if log.history_counter as usize >= log.log.len() {
            return;
        }
        log.history_counter += 1;
        commands.init_resource::<SetSelections>();
    } else if keys.any_pressed([KeyCode::ControlRight, KeyCode::ControlLeft])
        && keys.just_pressed(KeyCode::KeyZ)
        && keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
    {
        if log.history_counter == 0 {
            return;
        }
        log.history_counter -= 1;
        commands.init_resource::<SetSelections>();
    } else if keys.just_pressed(settings.hole_delete_button) {
        commands.init_resource::<HoleDelete>();
    } else if keys.just_pressed(settings.merge_delete_button) {
        commands.init_resource::<MergeDelete>();
    } else if keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keys.just_pressed(KeyCode::KeyR)
    {
        commands.init_resource::<ForceRefresh>();
    }
    // clear key presses after read
    keys.clear();
}

pub fn select_brush(
    mut commands: Commands,
    mut selection_plugin: ResMut<SelectionPluginSettings>,
    mut hover_reader: EventReader<Pointer<Over>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut s_query: Query<(Entity, &mut PickSelection)>,
    ui_res: Res<UiResource>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window.get_single_mut() {
        window.cursor.icon = match ui_res.cursor_enum {
            Cursor::Pointer => CursorIcon::Default,
            Cursor::Brush | Cursor::Eraser => CursorIcon::Crosshair,
        }
    }
    if ui_res.cursor_enum == Cursor::Pointer {
        return;
    }
    selection_plugin.click_nothing_deselect_all = false;
    commands.remove_resource::<EnablePanOrbit>();
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
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut pick_settings: ResMut<PickingPluginsSettings>,
    mut egui_context: Query<&mut EguiContext>,
) {
    let Ok(mut width) = egui_context.get_single_mut() else {
        return;
    };
    let width = width.get_mut().used_rect().width();
    let Ok(window) = window.get_single() else {
        return;
    };
    if let Some(Vec2 { x, .. }) = window.cursor_position() {
        if x < width {
            pick_settings.is_enabled = false;
            commands.remove_resource::<EnablePanOrbit>();
        }
    }
}

pub fn reset_ui_hover(
    mut commands: Commands,
    mut pick_settings: ResMut<PickingPluginsSettings>,
    ui_res: Res<UiResource>,
) {
    if ui_res.cursor_enum == Cursor::Pointer {
        commands.init_resource::<EnablePanOrbit>();
    }
    pick_settings.is_enabled = true;
}

#[derive(Default, Resource)]
pub struct EnablePanOrbit;
