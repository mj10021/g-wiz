use super::events::{UiEvent, UiEvent::ForceRefresh};
use crate::*;
use bevy::prelude::*;
use std::collections::HashSet; // Add this line to import the `SubdivideSelection` type

pub fn recalc_bounding_box(
    mut commands: Commands,
    mut bounding_box: ResMut<BoundingBox>,
    gcode: Res<GCode>,
) {
    bounding_box.recalculate(&gcode.0);
}

pub fn get_selections(mut s_query: Query<(&PickSelection, &Tag)>) -> HashSet<Id> {
    s_query
        .iter_mut()
        .filter_map(|(s, t)| if !s.is_selected { None } else { Some(t.id) })
        .collect()
}

pub fn merge_delete(
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    mut refresh: EventWriter<UiEvent>,
) {
    let mut selection = get_selections(s_query);
    gcode.0.merge_delete(&mut selection);
    refresh.send(ForceRefresh);
}

pub fn hole_delete(
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    mut refresh: EventWriter<UiEvent>,
) {
    let mut selection = get_selections(s_query);
    gcode.0.hole_delete(&mut selection);
    refresh.send(ForceRefresh);
}

pub fn subdivide(
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    mut refresh: EventWriter<UiEvent>,
) {
    let count = s_query.iter().len() as u32;
    let selection = get_selections(s_query);
    gcode.0.subdivide_vertices(selection, count);
    refresh.send(ForceRefresh);
}
