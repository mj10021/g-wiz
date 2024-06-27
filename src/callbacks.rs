use super::*;
use std::collections::HashSet;

fn get_selections(mut s_query: Query<(&PickSelection, &Tag)>) -> HashSet<Id> {
    s_query
        .iter_mut()
        .filter(|(s, _)| s.is_selected)
        .map(|(_, t)| t.id)
        .collect::<HashSet<Id>>()
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
