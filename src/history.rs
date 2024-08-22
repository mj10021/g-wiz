#![allow(dead_code)]
use super::{
    print_analyzer::{Parsed, Instruction, Vertex},
    GCode, Id, Resource, Tag,
};
use std::collections::{HashMap, HashSet};

fn vec_diff<T>(curr: &[T], next: &[T]) -> (bool, HashSet<(usize, T)>)
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
            gcode: gcode.0.clone()
        }
    }
    fn gcode_diff(&self, gcode: GCode) -> Diff {
        let line_diff = vec_diff(&gcode.0.lines, &self.gcode.lines);
        let vertex_diff = map_diff(&gcode.0.vertices, &self.gcode.vertices);
        Diff::GCodeDiff(line_diff, vertex_diff)
    }
    fn selection_diff(&self, selection: HashSet<Tag>) -> Diff {
        Diff::SelectionDiff(set_diff(&self.selections, &selection))
    }
}

enum Diff {
    GCodeDiff((bool, HashSet<(usize, Id)>), (bool, HashMap<Id, Vertex>)),
    SelectionDiff((bool, HashSet<Tag>))
}

#[derive(Resource)]
pub struct History {
    state: State,
    diff_log: Vec<Diff>,
    pub counter: usize,
}

impl History {
    fn get_diff(gcode: Res<GCode>, selections: Query<&PickSelection, &Tag>) {
        let gcode_diff = self.state.gcode_diff(gcode.0);
        let selection_diff = self.state.selection_diff(selections.iter().collect());
    }
    fn forward_apply(&mut self) {
        let cur = &self.diff_log[self.counter];
        match cur {
            Diff::GCodeDiff(line_diff, vertex_diff) => {
                let (dir, set) = line_diff;
                for (i, id) in set.iter() {
                    if *dir {
                        self.state.gcode.lines.insert(*i, *id);
                    } else {
                        self.state.gcode.lines.remove(*i);
                    }
                }
                let (dir, map) = vertex_diff;
                for (id, vertex) in map.iter() {
                    if *dir {
                        self.state.gcode.vertices.insert(*id, *vertex);
                    } else {
                        assert!(self.state.gcode.vertices.remove(id) == Some(*vertex)); // make sure the value is present
                    }
                }

            }
            Diff::SelectionDiff((dir, set)) => {
                if *dir {
                    self.state.selections.extend(set.iter());
                } else {
                    for tag in set.iter() {
                        assert!(self.state.selections.remove(tag)); // ensures removed value was present
                    }
                }
            }
        }

    }
    fn reverse_apply(&mut self) {}
}

