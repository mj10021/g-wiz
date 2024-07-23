use super::events::*;
use crate::{GCode, Tag, UiResource};
use bevy::prelude::*;
use bevy_mod_picking::selection::PickSelection;
pub enum Command<T: Default + bevy::prelude::Resource + Copy + Sized> {
    // t is the actual callback
    MergeDelete(T),
    HoleDelete(T),
    Subdivide(T),
    RecalcBounds(T),
    Undo(T),
    Redo(T),
}

fn ui_handler(mut event: EventReader<UiEvent>) {
    for event in event.read() {
        match event {
            UiEvent::ForceRefresh => {
                // this should run render
            }
            UiEvent::Undo => {
                // do something
            }
            UiEvent::Redo => {
                // do something
            }
            UiEvent::ExportDialogue => {
                // do something
            }
        }
    }
}


pub fn command_handler(
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    mut bounding_box: ResMut<crate::BoundingBox>,
    mut refresh: EventWriter<UiEvent>,
    mut event: EventReader<CommandEvent>,
    ui_res: Res<UiResource>,
) {
    let count = ui_res.subdivide_slider;
    let mut selection = s_query.iter().filter_map(f);
    for event in event.read() {
        match event {
            CommandEvent::MergeDelete => {
                gcode.0.merge_delete(&mut selection);
            }
            CommandEvent::HoleDelete => {
                gcode.0.hole_delete(&mut selection);

            }
            CommandEvent::Subdivide => {
                gcode.0.subdivide_vertices(selection.clone(), count);
            }
            CommandEvent::RecalcBounds => {
                bounding_box.recalculate(&gcode.0);
            }
            CommandEvent::Draw => {
                todo!();
            }
        }
        refresh.send(UiEvent::ForceRefresh);
    }
}
