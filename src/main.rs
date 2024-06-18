mod diff;
mod pan_orbit;
mod print_analyzer;
mod render;
mod ui;
mod select;

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_picking::prelude::*;
use diff::{undo_redo_selections, update_selection_log, SelectionLog, SetSelections};
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use picking_core::PickingPluginsSettings;
use print_analyzer::{Id, Parsed, Pos};
use render::*;
use selection::send_selection_events;
use std::collections::HashMap;
use std::env;
use ui::*;
use select::*;

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

fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
    let zoom = 35.0;
    let translation = Vec3::new(5.0 * zoom, -5.0 * zoom, 5.0 * zoom);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
    commands.init_resource::<ForceRefresh>();
    commands.init_resource::<UiResource>();
    commands.init_resource::<IdMap>();
    commands.init_resource::<EnablePanOrbit>();
    commands.init_resource::<SelectionLog>()
}
fn main() {
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
    App::new()
        .add_plugins((DefaultPlugins, DefaultPickingPlugins, EguiPlugin))
        .insert_resource(VertexCounter::build(&gcode))
        .insert_resource(GCode(gcode))
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
