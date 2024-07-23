use super::events::*;
use crate::{GCode, Tag, UiResource};
use bevy::{prelude::*, transform::commands};
use bevy_mod_picking::selection::PickSelection;

#[derive(Default, Resource)]
pub struct ForceRefresh;

fn ui_handler(
    mut event: EventReader<UiEvent>,
    mut commands: Commands,
    mut ui_res: ResMut<UiResource>,
) {
    for event in event.read() {
        match event {
            UiEvent::ForceRefresh => {
                commands.init_resource::<ForceRefresh>();
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
            UiEvent::MoveDisplay(forward, layer, count) => {
                if *layer && *forward {
                    ui_res.display_z_max.1 += count;
                    }
                else if *layer {
                        ui_res.display_z_max.0 += count;
                    }
                else if *forward {
                    ui_res.vertex_counter += *count as u32;
                } else {
                    if ui_res.vertex_counter == 0 {
                        return;
                    }
                    ui_res.vertex_counter -= *count as u32;
                }
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
    let mut selection = s_query
        .iter()
        .filter_map(|(s, t)| if !s.is_selected { None } else { Some(t.id) })
        .collect();
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
