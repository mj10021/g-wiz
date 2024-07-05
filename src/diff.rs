use crate::ForceRefresh;

use super::{
    print_analyzer::{Instruction, Vertex},
    GCode, Id, Resource, Tag,
};
use bevy::prelude::*;
use bevy_mod_picking::selection::PickSelection;
use std::collections::{HashMap, HashSet};

fn vec_diff<T>(curr: &Vec<T>, next: &Vec<T>) -> (bool, HashSet<(usize, T)>)
where
    T: Copy + Eq + std::hash::Hash,
{
    let mut out = HashSet::new();
    let mut i = 0;
    let add = curr.len() < next.len(); // add (true) if curr < len
    if add {
        for (j, elem) in next.iter().enumerate() {
            if i < curr.len() && curr[i] == next[j] {
                i += 1;
            } else {
                assert!(out.insert((j, *elem))); // make sure the inserted value is unique
            }
        }
    } else {
        for (j, elem) in curr.iter().enumerate() {
            if i < next.len() && next[i] == curr[j] {
                i += 1;
            } else {
                assert!(out.insert((j, *elem))); // make sure the inserted value is unique
            }
        }
    }
    (add, out)
}

fn set_diff<T>(curr: &HashSet<T>, next: &HashSet<T>) -> (bool, HashSet<T>)
where
    T: Copy + Eq + std::hash::Hash,
{
    if curr.len() < next.len() {
        (true, next.difference(curr).copied().collect::<HashSet<T>>())
    } else {
        (
            false,
            curr.difference(next).copied().collect::<HashSet<T>>(),
        )
    }
}

fn map_diff<S, T>(curr: &HashMap<S, T>, next: &HashMap<S, T>) -> (bool, HashMap<S, T>)
where
    S: Copy + PartialEq + Eq + core::hash::Hash,
    T: Clone,
{
    let add = curr.len() < next.len();
    let (curr_keys, next_keys) = (
        curr.keys().copied().collect::<HashSet<_>>(),
        next.keys().copied().collect::<HashSet<_>>(),
    );
    let diff_keys = {
        if add {
            curr_keys.difference(&next_keys)
        } else {
            next_keys.difference(&curr_keys)
        }
    }
    .collect::<HashSet<&S>>();
    let mut diff: HashMap<S, T> = HashMap::new();
    if add {
        for key in diff_keys.iter() {
            let value = next.get(*key).unwrap();
            diff.insert(**key, value.clone());
        }
    } else {
        for key in diff_keys.iter() {
            let value = curr.get(*key).unwrap();
            diff.insert(**key, value.clone());
        }
    }
    (add, diff)
}

#[derive(Resource)]
pub struct History {
    selection_log: SelectionLog,
    gcode_log: GCodeLog,
    log: Vec<bool>, // true means selection false means gcode
    pub history_counter: u32,
    curr_counter: u32,
    selection_counter: u32,
    gcode_counter: u32,
}

impl History {
    fn forward_apply(&mut self) {}
}

#[derive(Default, Resource)]
pub struct SelectionLog {
    curr: HashSet<Tag>,
    pub log: Vec<(bool, HashSet<Tag>)>,
    pub history_counter: u32,
    curr_counter: u32,
}

impl SelectionLog {
    fn diff(&self, next: &HashSet<Tag>) -> (bool, HashSet<Tag>) {
        set_diff(&self.curr, next)
    }

    fn forward_apply(&mut self, diff: (bool, HashSet<Tag>)) {
        let (add, diff) = diff;
        if add {
            self.curr.extend(diff.clone());
        } else {
            for elem in diff.iter() {
                assert!(self.curr.remove(elem)); // make sure element is actually removed
            }
        }
    }
    fn reverse_apply(&mut self, diff: (bool, HashSet<Tag>)) {
        let (add, diff) = diff;
        if add {
            for elem in diff.iter() {
                assert!(self.curr.remove(elem)); // make sure element is actually removed
            }
        } else {
            self.curr.extend(diff.clone())
        }
    }
}

// #[derive(Debug, PartialEq)]
// pub struct Parsed {
// pub lines: Vec<Id>,
// pub vertices: HashMap<Id, Vertex>,
// pub instructions: HashMap<Id, Instruction>,
// pub shapes: Vec<Shape>,
// pub rel_xyz: bool,
// pub rel_e: bool,
// id_counter: Id,
// }

#[derive(Resource)]
pub struct GCodeLog {
    curr: GCode,
    log: Vec<GCodeDiff>,
    pub history_counter: u32,
    curr_counter: u32,
}

impl GCodeLog {
    fn init(gcode: Res<GCode>) -> Self {
        Self {
            curr: gcode.clone(),
            log: Vec::new(),
            history_counter: 0,
            curr_counter: 0,
        }
    }

    fn diff(&self, next: &GCode) -> GCodeDiff {
        let line_diff = vec_diff(&self.curr.0.lines, &next.0.lines);
        let vertex_diff = map_diff(&self.curr.0.vertices, &next.0.vertices);
        let instruction_diff = map_diff(&self.curr.0.instructions, &next.0.instructions);
        GCodeDiff {
            add: line_diff.0,
            line_diff: line_diff.1,
            vertex_diff: vertex_diff.1,
            instruction_diff: instruction_diff.1,
        }
    }
}

pub struct GCodeDiff {
    add: bool,
    line_diff: HashSet<(usize, Id)>,
    vertex_diff: HashMap<Id, Vertex>,
    instruction_diff: HashMap<Id, Instruction>,
}

impl GCodeDiff {
    fn apply(&self, gcode: &mut GCode) {
        if self.add {
            for (i, id) in self.line_diff.iter() {
                gcode.0.lines.insert(*i as usize, *id);
            }
            gcode.0.vertices.extend(self.vertex_diff.clone());
            gcode.0.instructions.extend(self.instruction_diff.clone())
        } else {
            for (i, _id) in self.line_diff.iter() {
                gcode.0.lines.remove(*i);
            }
        }
        gcode.0.assign_shapes();
    }
}
#[derive(Resource, Default)]
pub struct SetSelections;

fn update_selection_log(
    mut commands: Commands,
    s_query: Query<(&PickSelection, &Tag)>,
    mut log: ResMut<SelectionLog>,
) {
    let new_set = s_query
        .iter()
        .filter(|(s, _)| s.is_selected)
        .map(|(_, t)| *t)
        .collect::<HashSet<Tag>>();
    let diff = log.diff(&new_set);
    if diff.1.is_empty() {
        return;
    }
    // if the counter isn't current and a the selection is made, clear the selection
    if log.history_counter != 0 {
        log.log = Vec::from([diff]);
        log.history_counter = 0;
        log.curr_counter = 0;
        log.curr = new_set;
        commands.remove_resource::<SetSelections>();
        return;
    }
    log.curr = new_set;
    log.log.push(diff);
    commands.init_resource::<SetSelections>()
}

fn update_gcode_log(mut commands: Commands, mut gcode: ResMut<GCode>, mut log: ResMut<GCodeLog>) {
    let diff = log.diff(&gcode);
    diff.apply(&mut gcode);
    log.log.push(diff);
    commands.init_resource::<ForceRefresh>();
}

pub fn update_logs(mut commands: Commands, mut gcode: ResMut<GCode>, s_query: Query<(&PickSelection, &Tag)>, mut history: ResMut<History>) {
    let gcode_diff = history.gcode_log.diff(&gcode);
    let selection_set = s_query
        .iter()
        .filter(|(s, _)| s.is_selected)
        .map(|(_, t)| *t)
        .collect::<HashSet<Tag>>();
    let selection_diff = history.selection_log.diff(&selection_set);
}

pub fn undo_redo_selections(
    mut commands: Commands,
    mut s_query: Query<(&mut PickSelection, &Tag)>,
    mut log: ResMut<SelectionLog>,
) {
    if log.log.is_empty() {
        return;
    }
    let mut updated = false;
    // ctrl+z
    while log.curr_counter < log.history_counter {
        let diff = log.log[log.log.len() - log.curr_counter as usize - 1].clone();
        log.reverse_apply(diff);
        log.curr_counter += 1;
        updated = true;
    }
    // ctrl+shift+z
    while log.curr_counter > log.history_counter {
        let diff = log.log[log.log.len() - log.curr_counter as usize].clone();
        log.forward_apply(diff);
        log.curr_counter -= 1;
        updated = true;
    }
    if updated {
        for (mut s, i) in s_query.iter_mut() {
            s.is_selected = log.curr.contains(i);
        }
    }
    commands.remove_resource::<SetSelections>();
}

//pub fn undo_redo_history(
//    mut commands: Commands,
//    mut s_query: Query<(&mut PickSelection, &Tag)>,
//    mut history: ResMut<History>,
//) {
//    if history.log.is_empty() {
//        return;
//    }
//    let mut updated = false;
//    // ctrl+z
//    while history.curr_counter < history.history_counter {
//
//        let selection = history.log[history.log.len() - history.curr_counter as usize - 1];
//        if selection {
//            history.selection_counter += 1;
//
//        } else {
////
//        }
//        log.reverse_apply(diff);
//        history.curr_counter += 1;
//        updated = true;
//    }
//    // ctrl+shift+z
//    while log.curr_counter > log.history_counter {
//        let diff = log.log[log.log.len() - log.curr_counter as usize].clone();
//        log.forward_apply(diff);
//        log.curr_counter -= 1;
//        updated = true;
//    }
//    if updated {
//        for (mut s, i) in s_query.iter_mut() {
//            s.is_selected = log.curr.contains(i);
//        }
//    }
//    commands.remove_resource::<SetSelections>()
//}
