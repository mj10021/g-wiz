mod callbacks;
mod diff;
mod pan_orbit;
mod print_analyzer;
mod render;
mod select;
mod settings;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::prelude::*;
use callbacks::*;
use diff::{undo_redo_selections, update_selection_log, SelectionLog, SetSelections};
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

#[derive(Default, Resource)]
struct ForceRefresh;

#[derive(Component, PartialEq, Copy, Clone, Hash, Eq, Debug)]
struct Tag {
    id: Id,
}

#[derive(Default, Resource)]
struct FilePath(String);

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

    commands.spawn((
        Camera3dBundle {
            ..Default::default()
        },
        PanOrbitCamera {
            ..Default::default()
        },
    ));
    commands.insert_resource(read_settings());
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
        .init_resource::<FilePath>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup, ui_setup, setup_render).chain())
        .add_systems(PreUpdate, select_erase_brush.before(send_selection_events))
        .add_systems(PreUpdate, capture_mouse.before(send_selection_events))
        .add_systems(
            Update,
            undo_redo_selections
                .run_if(resource_exists::<SetSelections>)
                .after(send_selection_events),
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
                export_dialogue.run_if(resource_exists::<ExportDialogue>),
                update_selections,
                update_visibilities,
                merge_delete.run_if(resource_exists::<MergeDelete>),
                hole_delete.run_if(resource_exists::<HoleDelete>),
                subdivide_selection.run_if(resource_exists::<SubdivideSelection>),
            )
                .chain(),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(resource_exists::<EnablePanOrbit>),
        )
        .add_systems(Update, render.run_if(resource_exists::<ForceRefresh>))
        .run();
}
