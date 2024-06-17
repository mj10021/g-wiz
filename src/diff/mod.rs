use bevy::prelude::*;
use bevy_mod_picking::selection::PickSelection;
use std::collections::HashSet;

use crate::ForceRefresh;

use super::{Resource, Tag};

#[derive(Default, Resource, Debug)]
pub struct SelectionLog {
    curr: HashSet<Tag>,
    pub log: Vec<SelectionDiff>,
    pub history_counter: u32,
    update_counter: u32,
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
    fn apply(curr: &mut HashSet<Tag>, diff: SelectionDiff) {
        curr.extend(diff.add);
        for elem in diff.sub.iter() {
            assert!(curr.remove(elem));
        }
    }
    fn is_some(&self) -> bool {
        !self.add.is_empty() || !self.sub.is_empty()
    }
}
#[derive(Resource, Default)]
pub struct SetSelections;
pub fn update_selection_log(mut commands: Commands, s_query: Query<(&PickSelection, &Tag)>, mut log: ResMut<SelectionLog>) {
    let new_set = s_query
        .iter()
        .filter(|(s, _)| s.is_selected)
        .map(|(_, t)| *t)
        .collect::<HashSet<Tag>>();
    let diff = SelectionDiff::diff(&log.curr, &new_set);
    if !diff.is_some() {
        return
    }
    // if log.history_counter > 0 {
    //     log.log = Vec::new();
    //     log.history_counter = 0;
    // }
    log.curr = new_set;
    log.log.push(diff);
    commands.init_resource::<SetSelections>()

}

pub fn undo_redo_selections(mut s_query: Query<(&mut PickSelection, &Tag)>, mut log: ResMut<SelectionLog>) {
    if log.update_counter == log.history_counter{return;}
    if log.history_counter == 0 {
        return;
    }
    log.log.reverse();
    let mut curr = log.curr.clone();
    for i in 0..log.history_counter as usize {
        SelectionDiff::apply(&mut curr, log.log[i].clone());
        println!("{:?}", log);
    }
    log.log.reverse();

    for (mut s, i) in s_query.iter_mut() {
        s.is_selected = curr.contains(i);
    }
    log.update_counter = log.history_counter;
}