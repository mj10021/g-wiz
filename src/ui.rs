use crate::print_analyzer::Parsed;
use crate::{ForceRefresh, GCode, Tag};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use bevy_mod_picking::selection::PickSelection;
use std::collections::HashSet;

#[derive(PartialEq, Clone, Copy)]
pub enum Choice {
    Vertex,
    Shape,
    Layer,
}

#[derive(Resource)]
pub struct UiResource {
    layer_counter: u32,
    pub vertex_counter: u32,
    pub selection_enum: Choice,
    subdivide_slider: f32,
    translation_input: String,
    pub panel_size: (f32, f32),
    insert_text: String,
    pub gcode_emit: String,
    pub vis_select: VisibilitySelector,
}

impl Default for UiResource {
    fn default() -> Self {
        UiResource {
            layer_counter: 0,
            vertex_counter: 0,
            selection_enum: Choice::Vertex,
            subdivide_slider: 100.0,
            translation_input: String::new(),
            panel_size: (0.0, 0.0),
            insert_text: String::new(),
            gcode_emit: String::new(),
            vis_select: VisibilitySelector::default(),
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

pub fn ui_example_system(
    mut contexts: EguiContexts,
    mut commands: Commands,
    vertex: Res<VertexCounter>,
    layer: Res<LayerCounter>,
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
    ui_res.panel_size = (panel_width, height);
    let height = window.height();
    let spacing = height / 50.0;
    let max = vertex.max;
    let layer_max = layer.max;
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
            ui.label("world");
            ui.add_space(spacing);
            ui.add(egui::Slider::new(&mut ui_res.vertex_counter, 0..=max));
            ui.add_space(spacing);
            ui.add(egui::Slider::new(&mut ui_res.layer_counter, 0..=layer_max).vertical());
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
                ui.radio_value(&mut ui_res.selection_enum, Choice::Vertex, "Vertex");
                ui.radio_value(&mut ui_res.selection_enum, Choice::Shape, "Shape");
                ui.radio_value(&mut ui_res.selection_enum, Choice::Layer, "Layer");
            });
            ui.add_space(spacing);
            ui.horizontal(|ui| {
                if ui.button("Merge Delete").clicked() {
                    gcode.0.delete_lines(&mut selection)
                } else if ui.button("Hole Delete").clicked() {
                    todo!();
                }
            });
            ui.add_space(spacing);
            ui.horizontal(|ui| {
                let _response = ui.add(egui::Slider::new(&mut ui_res.subdivide_slider, 0.0..=30.0));
                if ui.button("Subdivide to max distance").clicked() {
                    gcode.0.subdivide_all(ui_res.subdivide_slider);
                    commands.insert_resource(ForceRefresh);
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
                                let v = gcode.0.vertices.get_mut(selection).unwrap();
                                v.to.x += x;
                                v.to.y += y;
                                v.to.z += z;
                            }
                        }
                        Choice::Shape => {
                            let mut shapes = HashSet::new();
                            for selection in &selection {
                                let shape = gcode.0.get_shape(selection);
                                shapes.extend(&shape);
                            }
                            for vertex in shapes.iter() {
                                let v = gcode.0.vertices.get_mut(vertex).unwrap();
                                v.to.x += x;
                                v.to.y += y;
                                v.to.z += z;
                            }
                        }
                        Choice::Layer => {
                            let mut layers = HashSet::new();
                            for selection in &selection {
                                let layer = gcode.0.get_layer(selection);
                                layers.extend(&layer);
                            }
                            for vertex in layers.iter() {
                                let v = gcode.0.vertices.get_mut(vertex).unwrap();
                                v.to.x += x;
                                v.to.y += y;
                                v.to.z += z;
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
            ui.horizontal(|ui| {
                let _response = ui.text_edit_singleline(&mut ui_res.insert_text);
                if ui.button("Insert Before").clicked() {
                    gcode.0.insert_before(&ui_res.insert_text, &selection)
                }
            });
            ui.add_space(spacing);
            ui.text_edit_multiline(&mut ui_res.gcode_emit)
                .on_hover_text("enter custom gcode");
            ui.add_space(spacing);
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
#[derive(Resource)]
pub struct LayerCounter {
    max: u32,
}

impl LayerCounter {
    pub fn build(gcode: &Parsed) -> LayerCounter {
        LayerCounter {
            max: gcode.layers.len() as u32,
        }
    }
}

pub fn key_system(mut ui_res: ResMut<UiResource>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.pressed(KeyCode::ArrowLeft) {
        ui_res.vertex_counter -= 1;
    } else if keys.pressed(KeyCode::ArrowRight) {
        ui_res.vertex_counter += 1;
    } else if keys.pressed(KeyCode::ArrowUp) {
        ui_res.layer_counter += 1;
    } else if keys.pressed(KeyCode::ArrowDown) {
        ui_res.layer_counter -= 1;
    }
}
