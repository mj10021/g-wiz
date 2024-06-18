use super::{Resource, Tag};
use bevy::prelude::*;
use bevy_mod_picking::selection::PickSelection;
use std::collections::HashSet;

#[derive(Default, Resource, Debug)]
pub struct SelectionLog {
    curr: HashSet<Tag>,
    pub log: Vec<SelectionDiff>,
    pub history_counter: u32,
    curr_counter: u32,
}

#[derive(Clone, Debug)]
pub struct SelectionDiff {
    add: HashSet<Tag>,
    sub: HashSet<Tag>,
}

impl SelectionDiff {
    fn diff(curr: &HashSet<Tag>, next: &HashSet<Tag>) -> SelectionDiff {
        let sub = curr.difference(next).copied().collect::<HashSet<_>>();
        let add = next.difference(curr).copied().collect::<HashSet<_>>();
        SelectionDiff { add, sub }
    }
    fn forward_apply(&self, curr: &mut HashSet<Tag>) {
        curr.extend(self.add.clone());
        for elem in self.sub.iter() {
            assert!(curr.remove(elem));
        }
    }
    fn reverse_apply(&self, curr: &mut HashSet<Tag>) {
        curr.extend(self.sub.clone());
        for elem in self.add.iter() {
            assert!(curr.remove(elem));
        }
    }
    fn is_none(&self) -> bool {
        self.add.is_empty() && self.sub.is_empty()
    }
}
#[derive(Resource, Default)]
pub struct SetSelections;
pub fn update_selection_log(
    mut commands: Commands,
    s_query: Query<(&PickSelection, &Tag)>,
    mut log: ResMut<SelectionLog>,
) {
    let new_set = s_query
        .iter()
        .filter(|(s, _)| s.is_selected)
        .map(|(_, t)| *t)
        .collect::<HashSet<Tag>>();
    let diff = SelectionDiff::diff(&log.curr, &new_set);
    if diff.is_none() {
        return;
    }
    // if log.history_counter > 0 {
    //     log.log = Vec::new();
    //     log.history_counter = 0;
    // }
    log.curr = new_set;
    log.log.push(diff);
    commands.init_resource::<SetSelections>()
}

pub fn undo_redo_selections(
    mut commands: Commands,
    mut s_query: Query<(&mut PickSelection, &Tag)>,
    mut log: ResMut<SelectionLog>,
) {
    let mut updated = false;
    // ctrl+z
    while log.curr_counter < log.history_counter {
        let diff = log.log[log.log.len() - log.curr_counter as usize - 1].clone();
        diff.reverse_apply(&mut log.curr);
        log.curr_counter += 1;
        updated = true;
    }
    // ctrl+shift+z
    while log.curr_counter > log.history_counter {
            let diff = log.log[log.log.len() - log.curr_counter as usize].clone();
            diff.forward_apply(&mut log.curr);
        log.curr_counter -= 1;
        updated = true;
    }
    if updated {
        for (mut s, i) in s_query.iter_mut() {
            s.is_selected = log.curr.contains(i);
        }
    }
    commands.remove_resource::<SetSelections>()
}
