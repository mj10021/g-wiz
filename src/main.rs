mod diff;
mod pan_orbit;
mod print_analyzer;
mod render;
mod select;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_picking::prelude::*;
use diff::{undo_redo_selections, update_selection_log, SelectionLog, SetSelections};
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use picking_core::PickingPluginsSettings;
use print_analyzer::{Id, Parsed, Pos};
use render::*;
use select::*;
use selection::send_selection_events;
use std::collections::HashMap;
use std::env;
use ui::*;

#[derive(Default, Resource)]
struct IdMap(HashMap<Id, Entity>);

#[derive(Resource)]
struct GCode(Parsed);

#[derive(Default, Resource)]
struct ForceRefresh;

#[derive(Component, PartialEq, Copy, Clone, Hash, Eq, Debug)]
struct Tag {
    id: Id,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let args: Vec<String> = env::args().collect();

    // Check if a filename was provided
    let filename: &str;
    if args.len() < 2 {
        println!("invalid file provided, opening test cube instead");
        filename = "../print_analyzer/test.gcode";
    } else {
        let name = &args[1];
        if name == "goblin" {
            filename = "../print_analyzer/Goblin Janitor_0.4n_0.2mm_PLA_MINIIS_10m.gcode";
        } else {
            filename = name;
        }
    }
    let gcode = print_analyzer::read(filename, false).expect("failed to read");
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
    let translation = Vec3::new(-360.0, -360.0, 346.53);
    let rotation = Quat::from_vec4(Vec4::new(0.44708318, -0.0770713, -0.30120307, 0.83872086));
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform {
                translation,
                rotation,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
    let (w, l, h) = (300.0, 300.0, 300.0);
    let _ = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(w, l, -0.1)),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: Color::WHITE,
            ..Default::default()
        }),
        transform: Transform {
            translation: Vec3::new(w / 2.0, l / 2.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
    let step = 25;
    for x in (0..=w as i32).step_by(step) {
        for y in (0..=l as i32).step_by(step) {
            let (x, y) = (x as f32, y as f32);
            let _ = commands.spawn(PbrBundle {
                mesh: meshes.add(
                    Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD)
                        .with_inserted_attribute(
                            Mesh::ATTRIBUTE_POSITION,
                            Vec::from([Vec3::new(x, y, 0.0), Vec3::new(x, y, 300.0)]),
                        ),
                ),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
    }

    commands.insert_resource(VertexCounter::build(&gcode));
    commands.insert_resource(GCode(gcode));
    commands.init_resource::<ForceRefresh>();
    commands.init_resource::<UiResource>();
    commands.init_resource::<IdMap>();
    commands.init_resource::<EnablePanOrbit>();
    commands.init_resource::<SelectionLog>();
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
        ))
        .add_systems(Startup, (setup, ui_setup).chain())
        .add_systems(PreUpdate, capture_mouse.before(send_selection_events))
        .add_systems(
            Update,
            undo_redo_selections
                .run_if(resource_exists::<SetSelections>)
                .after(send_selection_events),
        )
        .add_systems(
            Update,
            (
                select_brush,
                key_system,
                ui_system,
                update_selections,
                update_visibilities,
            )
                .chain(),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(resource_exists::<EnablePanOrbit>),
        )
        .add_systems(Update, render.run_if(resource_exists::<ForceRefresh>))
        .add_systems(PostUpdate, (reset_ui_hover, update_selection_log))
        .run();
}
