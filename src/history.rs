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
    
struct State {
    selections: HashSet<Tag>,
    gcode: Parsed,
}

impl State {
    fn build(gcode: GCode) -> Self {
        Self {
            selections: HashSet::new(),
            gcode: gcode.clone()
        }
    }
    fn gcode_diff(&self, gcode: GCode) -> ((bool, HashSet<(usize, Id)>), (bool, HashMap<Id, Vertex>)) {
        let line_diff = vec_diff(gcode.lines, self.gcode.lines);
        let vertex_diff = map_diff(gcode.vertices, self.gcode.vertices);
        (line_diff, vertex_diff)
}

#[derive(Resource)]
pub struct History {
    state: State,
    diff_log: Vec<StateDiff>,
    pub counter: u32,
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

#[derive(Resource)]
pub struct GCodeLog {
    curr: GCode,
    log: Vec<GCodeDiff>,
    pub history_counter: u32,
    curr_counter: u32,
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

