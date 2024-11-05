mod events;
mod models;
mod history;
mod pan_orbit;
mod render;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::prelude::*;
use diff::Diff;
use events::{console::*, handlers::*, *};
use g_win::{GCodeModel, Id};
use models::{Label, State, VertexMap};
use history::*;
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use picking_core::PickingPluginsSettings;
use render::*;
use selection::send_selection_events;
use settings::*;
use std::collections::HashMap;
use std::env;
use ui::*;

#[derive(Default, Resource)]
struct IdMap {
    pub id_to_entity: HashMap<Id, Entity>,
    pub entity_to_id: HashMap<Entity, Id>,
}

impl IdMap {
    fn insert(&mut self, id: Id, entity: Entity) {
        self.id_to_entity.insert(id, entity);
        self.entity_to_id.insert(entity, id);
    }
    fn remove(&mut self, id: Id, entity: Entity) {
        assert_eq!(self.id_to_entity.remove(&id), Some(entity));
        assert_eq!(self.entity_to_id.remove(&entity), Some(id));
    }
}

#[derive(Clone, Resource)]
struct GCode(GCodeModel);

#[derive(Component, PartialEq, Copy, Clone, Hash, Eq, Debug)]
struct Tag {
    id: Id,
}

#[derive(Default, Resource)]
struct FilePath(std::path::PathBuf);

#[derive(Debug, Resource)]
pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    fn from(vertices: &VertexMap) -> Self {
        let mut out = Self {
            min: Vec3::INFINITY,
            max: Vec3::NEG_INFINITY,
        };
        for v in vertices.0.values() {
            if v.label != Label::Extrusion {
                continue;
            }
            let (x, y, z) = (v.x(), v.y(), v.z());
            out.min.x = out.min.x.min(x);
            out.min.y = out.min.y.min(y);
            out.min.z = out.min.z.min(z);
            out.max.x = out.max.x.max(x);
            out.max.y = out.max.y.max(y);
            out.max.z = out.max.z.max(z);
        }
        out
    }
    pub fn recalculate(&mut self, vertices: &VertexMap) {
        for v in vertices.0.values() {
            if v.label != Label::Extrusion {
                continue;
            }
            let (x, y, z) = (v.x(), v.y(), v.z());
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

fn setup(mut commands: Commands, mut filepath: ResMut<FilePath>) {
    let args: Vec<String> = env::args().collect();
    let default = "./";

    // Check if a filename was provided
    let filename = {
        if args.len() < 2 {
            println!("invalid file provided, opening demo");
            default
        } else {
            &args[1]
        }
    };
    filepath.0 = filename.parse().unwrap_or(default.parse().unwrap());
    let gcode = GCodeModel::from_file(&filepath.0).unwrap_or(
        crate::settings::DEFAULT_GCODE
            .parse()
            .expect("default gcode failed to parse"),
    );
    let vertices = VertexMap::build(&gcode);
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 255.0,
    });
    let bounding_box = BoundingBox::from(&vertices);
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
    commands.insert_resource(VertexCounter::build(&vertices));
    commands.insert_resource(History::build(&gcode, &vertices));
    commands.insert_resource(GCode(gcode));
    commands.init_resource::<UiResource>();
    commands.init_resource::<IdMap>();
    commands.init_resource::<PanOrbit>();
    commands.init_resource::<ForceRefresh>();
    commands.init_resource::<Console>();
    commands.init_resource::<ConsoleActive>();
    commands.init_resource::<ExportDialogue>();
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
        .add_event::<SystemEvent>()
        .init_resource::<FilePath>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup, ui_setup, setup_render).chain())
        .add_systems(PreUpdate, select_erase_brush.before(send_selection_events))
        .add_systems(PreUpdate, (capture_mouse).before(send_selection_events))
        .add_systems(PreUpdate, update_history_diff_log)
        // .add_systems(Update, update_selection_log.before(undo_redo_selections))
        .add_systems(
            Update,
            (
                right_click,
                key_system,
                toolbar,
                right_click_menu.run_if(resource_exists::<RightClick>),
                sidebar,
                console,
                selection_handler,
                update_visibilities,
                ui_handler,
                command_handler,
                system_handler,
            )
                .chain(),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(resource_equals::<PanOrbit>(PanOrbit(true))),
        )
        .add_systems(
            PostUpdate,
            (render.run_if(resource_exists::<ForceRefresh>), undo_redo).chain(),
        )
        .run();
}
