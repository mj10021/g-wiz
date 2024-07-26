use super::events::*;
use crate::{GCode, PanOrbit, Tag, UiResource, IdMap, Choice};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

#[derive(Default, Resource)]
pub struct ForceRefresh;

pub fn ui_handler(
    mut event: EventReader<UiEvent>,
    mut commands: Commands,
    mut pan_orbit: ResMut<PanOrbit>,
    mut ui_res: ResMut<UiResource>,
    mut s_query: Query<&mut PickSelection>,
) {
    for event in event.read() {
        match event {
            UiEvent::ForceRefresh => {
                commands.init_resource::<ForceRefresh>();
            }
            UiEvent::MoveDisplay(forward, layer, count) => {
                if *layer && *forward {
                    ui_res.display_z_max.1 += count;
                } else if *layer {
                    ui_res.display_z_max.0 += count;
                } else if *forward {
                    ui_res.vertex_counter += *count as u32;
                } else {
                    if ui_res.vertex_counter == 0 {
                        return;
                    }
                    ui_res.vertex_counter -= *count as u32;
                }
            }
            UiEvent::SelectAll => {
                let select_all = s_query.iter().any(|s| !s.is_selected);
                for mut selection in s_query.iter_mut() {
                    selection.is_selected = select_all;
                }
            }
            UiEvent::SetPanOrbit(on) => {
                pan_orbit.0 = *on;
            }
            UiEvent::ConsoleEnter(s) => todo!(),
            UiEvent::ConsoleResponse(s) => todo!(),
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
    let centroid = gcode.0.get_centroid(&selection);
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
            CommandEvent::InsertPoint => {
                todo!();
            }
            CommandEvent::Translate(vec) => {
                for id in selection.iter() {
                    gcode.0.translate(id, vec);
                }
            }
            CommandEvent::Rotate(vec) => {
                for id in selection.iter() {
                    gcode.0.rotate(id, &centroid, vec);
                }
            }
            CommandEvent::Scale(vec) => {
                for id in selection.iter() {
                    gcode.0.scale(id, &centroid, vec);
                }
            }
            CommandEvent::Undo => {
                // do something
            }
            CommandEvent::Redo => {
                // do something
            }
        }
        refresh.send(UiEvent::ForceRefresh);
    }
}

pub fn selection_handler(
    mut select_reader: EventReader<Pointer<Select>>,
    mut deselect_reader: EventReader<Pointer<Deselect>>,
    ui_res: Res<UiResource>,
    mut selectables: Query<(&mut PickSelection, &Tag)>,
    map: Res<IdMap>,
    gcode: Res<GCode>,
) {
    let select_ids = select_reader
        .read()
        .map(|s| selectables.get(s.target).unwrap().1.id);
    let deselect_ids = deselect_reader
        .read()
        .map(|s| &selectables.get(s.target).unwrap().1.id);
    let events = {
        let mut out = (Vec::new(), Vec::new());
        match ui_res.selection_enum {
            Choice::Vertex => {}
            Choice::Shape => {
                for id in select_ids {
                    out.0.append(&mut gcode.0.get_shape(&id));
                }
                for id in deselect_ids {
                    out.1.append(&mut gcode.0.get_shape(&id));
                }
            }
            Choice::Layer => {
                for id in select_ids {
                    out.0.append(&mut gcode.0.get_same_z(&id));
                }
                for id in deselect_ids {
                    out.1.append(&mut gcode.0.get_same_z(&id));
                }
            }
        }
        out
    };
    for id in events.0 {
        if let Some(entity) = map.0.get(&id) {
            if let Ok((mut select_me, _)) = selectables.get_mut(*entity) {
                select_me.is_selected = true;
            }
        }
    }
    for id in events.1 {
        if let Some(entity) = map.0.get(&id) {
            if let Ok((mut select_me, _)) = selectables.get_mut(*entity) {
                select_me.is_selected = false;
            }
        }
    }
}
