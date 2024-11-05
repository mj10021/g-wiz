use crate::{models::GeometryMap, Diff, GCode, GCodeModel, Resource, Tag};
use bevy::prelude::*;
use bevy_mod_picking::prelude::PickSelection;
use std::collections::HashSet;

#[derive(Default, Resource)]
pub struct History {
    pub counter: usize,
    pub counter_cur: usize,
    pub diff_log: Vec<usize>,
}
pub fn undo_redo(
    mut history: ResMut<History>,
    mut gcode: ResMut<GCode>,
    mut state: ResMut<State>,
    mut selections: Query<(&mut PickSelection, &Tag)>,
) {
    while history.counter != history.counter_cur {
        if history.counter_cur > history.counter {
            history.apply_current(true);
            history.apply_to_gcode(&mut gcode.0, &mut vertices, true);
            history.counter_cur -= 1;
        } else {
            history.apply_current(false);
            history.apply_to_gcode(&mut gcode.0, &mut vertices, false);
            history.counter_cur += 1;
        }
        // apply state selections to selection query
        for (mut selection, tag) in selections.iter_mut() {
            selection.is_selected = history.state.selections.contains(tag);
        }
    }
}
