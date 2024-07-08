use super::*;
use std::collections::HashSet;

#[derive(Default, Resource)]
pub struct MergeDelete;

#[derive(Default, Resource)]
pub struct HoleDelete;

#[derive(Default, Resource)]
pub struct SubdivideSelection(pub u32);

fn get_selections(mut s_query: Query<(&PickSelection, &Tag)>) -> HashSet<Id> {
    s_query
        .iter_mut()
        .filter_map(|(s, t)| if !s.is_selected { None } else { Some(t.id) })
        .collect()
}

pub fn merge_delete(
    mut commands: Commands,
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
) {
    let mut selection = get_selections(s_query);
    gcode.0.merge_delete(&mut selection);
    commands.init_resource::<ForceRefresh>();
    commands.remove_resource::<MergeDelete>();
}

pub fn hole_delete(
    mut commands: Commands,
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
) {
    let mut selection = get_selections(s_query);
    gcode.0.hole_delete(&mut selection);
    commands.init_resource::<ForceRefresh>();
    commands.remove_resource::<HoleDelete>();
}

pub fn subdivide_selection(
    mut commands: Commands,
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    count: Res<SubdivideSelection>,
) {
    let selection = get_selections(s_query);
    gcode.0.subdivide_vertices(selection, count.0);
    commands.init_resource::<ForceRefresh>();
    commands.remove_resource::<SubdivideSelection>();
}
