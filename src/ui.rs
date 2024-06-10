use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use print_analyzer::Parsed;

use crate::{GCode, Selection};

#[derive(PartialEq, Clone, Copy)]
enum Choice {
    Vertex,
    Shape,
    Layer,
}

#[derive(Resource)]
pub struct Enum(Choice);

impl Default for Enum {
    fn default() -> Self {
        Enum(Choice::Vertex)
    }
}


pub fn ui_example_system(
    mut contexts: EguiContexts,
    vertex: Res<VertexCounter>,
    layer: Res<LayerCounter>,
    mut counter: ResMut<SecretCount>,
    mut layer_counter: ResMut<SecretLayerCount>,
    mut enu: ResMut<Enum>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window.get_single() else {panic!(); };
    let height = window.height();
    let spacing = height / 50.0;
    let max = vertex.max;
    let layer_max = layer.max;
    egui::SidePanel::new(egui::panel::Side::Left, "panel").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.add_space(spacing);
        ui.add(egui::Slider::new(&mut counter.0, 0..=max));
        ui.add_space(spacing);
        ui.add(egui::Slider::new(&mut layer_counter.0, 0..=layer_max).vertical());
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
                            counter.0 -= num;
                        } else {
                            counter.0 += num;
                        }
                    }
                    i += 1;
                }
            });
        ui.add_space(spacing);
        ui.horizontal(|ui| {
            ui.radio_value(&mut enu.0, Choice::Vertex, "Vertex");
            ui.radio_value(&mut enu.0, Choice::Shape, "Shape");
            ui.radio_value(&mut enu.0, Choice::Layer, "Layer");
        });
        ui.add_space(spacing);
        ui.horizontal(|ui| {
            if ui.button("Merge Delete").clicked() {
                todo!();
            }
            else if ui.button("Hole Delete").clicked() {
                todo!();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Subdivide").clicked() {
                todo!();
            }

        });
        // ui.horizontal(|ui| {
        //     let _response = ui.text_edit_singleline(&mut func.0);

        //     let enu = enu.0;
        //     if ui.button("Translate").clicked() {
        //         let mut params = func.0.split_whitespace();
        //         let x = params.next().unwrap().parse::<f32>().unwrap();
        //         let y = params.next().unwrap().parse::<f32>().unwrap();
        //         let z = params.next().unwrap().parse::<f32>().unwrap();
        //         match enu {
        //             Choice::Vertex => {
        //                 let v = gcode.0.vertices.get_mut(&selection.0).unwrap();
        //                 v.to.x += x;
        //                 v.to.y += y;
        //                 v.to.z += z;
        //             },
        //             Choice::Shape => {
        //                 let shapes = gcode.0.get_shape(&selection.0);
        //                 for vertex in shapes.iter() {
        //                     let v = gcode.0.vertices.get_mut(vertex).unwrap();
        //                     v.to.x += x;
        //                     v.to.y += y;
        //                     v.to.z += z;
        //                 }
        //             },
        //             Choice::Layer => {
        //                 let layer = gcode.0.get_layer(&selection.0);
        //                 for vertex in layer.iter() {
        //                     let v = gcode.0.vertices.get_mut(vertex).unwrap();
        //                     v.to.x += x;
        //                     v.to.y += y;
        //                     v.to.z += z;
        //                 }
        //             }
        //         }
        //     }
        // });
    });
}
pub fn update_count(secret: Res<SecretCount>, mut counter: ResMut<VertexCounter>) {
    if secret.0 as u32 != counter.count {
        counter.count = secret.0 as u32;
    }
}

#[derive(Resource)]
pub struct VertexCounter {
    pub count: u32,
    max: u32,
}

impl VertexCounter {
    pub fn build(gcode: &Parsed) -> VertexCounter {
        let max = gcode.vertices.keys().len() as u32;
        VertexCounter { count: 0, max }
    }
}
#[derive(Resource)]
pub struct LayerCounter {
    count: u32,
    max: u32,
}

impl LayerCounter {
    pub fn build(gcode: &Parsed) -> LayerCounter {
        let max = gcode.layers.len() as u32;
        LayerCounter { count: 0, max }
    }
}
#[derive(Resource)]
pub struct SecretCount(pub u32);
impl Default for SecretCount {
    fn default() -> Self {
        SecretCount(0)
    }
}

#[derive(Resource)]
pub struct SecretLayerCount(pub u32);
impl Default for SecretLayerCount {
    fn default() -> Self {
        SecretLayerCount(0)
    }
}

pub fn key_system(
    mut counter: ResMut<SecretCount>,
    mut layer_counter: ResMut<SecretLayerCount>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.pressed(KeyCode::ArrowLeft) {
        counter.0 -= 1;
    } else if keys.pressed(KeyCode::ArrowRight) {
        counter.0 += 1;
    } else if keys.pressed(KeyCode::ArrowUp) {
        layer_counter.0 += 1;
    } else if keys.pressed(KeyCode::ArrowDown) {
        layer_counter.0 -= 1;
    }
}
