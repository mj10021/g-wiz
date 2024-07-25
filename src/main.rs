mod callbacks;
mod history;
mod load_assets;
mod pan_orbit;
mod print_analyzer;
mod render;
mod select;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::prelude::*;
use callbacks::events::*;
use callbacks::handlers::*;
use history::{undo_redo_selections, update_selection_log, SelectionLog};
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use picking_core::PickingPluginsSettings;
use print_analyzer::{Id, Parsed};
use render::*;
use select::*;
use selection::send_selection_events;
use settings::*;
use std::collections::HashMap;
use std::env;
use ui::*;

#[derive(Default, Resource)]
struct IdMap(HashMap<Id, Entity>);

#[derive(Clone, Resource)]
struct GCode(Parsed);

#[derive(Component, PartialEq, Copy, Clone, Hash, Eq, Debug)]
struct Tag {
    id: Id,
}

#[derive(Default, Resource)]
struct FilePath(String);

#[derive(Debug, Resource)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    fn from(gcode: &Parsed) -> Self {
        let mut out = Self {
            min: Vec3::INFINITY,
            max: Vec3::NEG_INFINITY,
        };
        for v in gcode.vertices.values() {
            if !v.extrusion_move() {
                continue;
            }
            let (x, y, z) = (v.to.x, v.to.y, v.to.z);
            out.min.x = out.min.x.min(x);
            out.min.y = out.min.y.min(y);
            out.min.z = out.min.z.min(z);
            out.max.x = out.max.x.max(x);
            out.max.y = out.max.y.max(y);
            out.max.z = out.max.z.max(z);
        }
        out
    }
    pub fn recalculate(&mut self, gcode: &Parsed) {
        for v in gcode.vertices.values() {
            if !v.extrusion_move() {
                continue;
            }
            let (x, y, z) = (v.to.x, v.to.y, v.to.z);
            self.min.x = self.min.x.min(x);
            self.min.y = self.min.y.min(y);
            self.min.z = self.min.z.min(z);
            self.max.x = self.max.x.max(x);
            self.max.y = self.max.y.max(y);
            self.max.z = self.max.z.max(z);
        }
    }
    fn midpoint(&self) -> Vec3 {
        Vec3 {
            x: (self.max.x - self.min.x) / 2.0,
            y: (self.max.y - self.min.y) / 2.0,
            z: (self.max.z - self.min.z) / 2.0,
        }
    }
}

    fn centroid(vertices: &Vec<Vec3>) -> Vec3 {
    let (mut i, mut j, mut k) = (0.0, 0.0, 0.0);
    let count = vertices.len() as f32;
    for Vec3 { x, y, z } in vertices {
        i += x;
        j += y;
        k += z;
    }
    Vec3 {
        x: i / count,
        y: j / count,
        z: k / count,
    }
}

fn setup(mut commands: Commands, mut filepath: ResMut<FilePath>) {
    let args: Vec<String> = env::args().collect();

    // Check if a filename was provided
    let filename = {
        if args.len() < 2 {
            println!("invalid file provided, opening demo");
            "./"
        } else {
            &args[1]
        }
    };
    filepath.0 = filename.to_string();
    let gcode = print_analyzer::read(filename, false)
        .unwrap_or(print_analyzer::read(crate::settings::DEFAULT_GCODE, true).unwrap());
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 255.0,
    });
    let bounding_box = BoundingBox::from(&gcode);
    let center = bounding_box.midpoint();
    let transform = Transform::from_xyz(bounding_box.min.x - center.x, center.y, 200.0)
        .looking_at(center, Vec3::Y);
    let radius = transform.translation.distance(center);
    commands.spawn((
        Camera3dBundle {
            transform,
            ..Default::default()
        },
        PanOrbitCamera {
            focus: center,
            radius,
            ..Default::default()
        },
    ));
    commands.insert_resource(bounding_box);
    commands.insert_resource(read_settings());
    commands.insert_resource(VertexCounter::build(&gcode));
    commands.insert_resource(GCode(gcode));
    commands.init_resource::<UiResource>();
    commands.init_resource::<IdMap>();
    commands.init_resource::<PanOrbit>();
    commands.init_resource::<SelectionLog>();
    commands.init_resource::<SelectAll>();
    commands.init_resource::<ForceRefresh>();
}
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    mode: bevy::window::WindowMode::Windowed,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            DefaultPickingPlugins,
            EguiPlugin,
            MaterialPlugin::<LineMaterial>::default(),
        ))
        .add_event::<UiEvent>()
        .add_event::<CommandEvent>()
        .add_event::<FileEvent>()
        .init_resource::<FilePath>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup, ui_setup, setup_render).chain())
        .add_systems(PreUpdate, select_erase_brush.before(send_selection_events))
        .add_systems(PreUpdate, (capture_mouse).before(send_selection_events))
        .add_systems(
            PreUpdate,
            select_deselect_all.run_if(resource_changed::<SelectAll>),
        )
        .add_systems(Update, update_selection_log.before(undo_redo_selections))
        .add_systems(
            Update,
            (
                right_click,
                key_system,
                toolbar,
                right_click_menu.run_if(resource_exists::<RightClick>),
                ui_system,
                console,
                export_dialogue.run_if(resource_exists::<ExportDialogue>),
                update_selections,
                update_visibilities,
                ui_handler,
                command_handler,
            )
                .chain(),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(resource_equals::<PanOrbit>(PanOrbit(true))),
        )
        .add_systems(Update, render.run_if(resource_exists::<ForceRefresh>))
        .run();
}
