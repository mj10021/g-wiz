use super::{
    parser::{Parsed, Vertex},
    GCode, Id, Resource, Tag,
};
use bevy::prelude::*;
use bevy_mod_picking::prelude::PickSelection;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Copy, Clone, Debug)]
pub enum DiffType {
    Add,
    Remove,
    Modify,
}

fn vec_diff<T>(curr: &[T], next: &[T]) -> (bool, HashSet<(usize, T)>)
// FIXME: make sure the indicies are sorted so that the right lines are removed
where
    T: Clone + Eq + std::hash::Hash,
{
    let mut out = HashSet::new();
    let mut i = 0;
    let add = curr.len() < next.len(); // add (true) if curr < len
    if add {
        for (j, elem) in next.iter().enumerate() {
            if i < curr.len() && curr[i] == next[j] {
                i += 1;
            } else {
                assert!(out.insert((j, elem.clone()))); // make sure the inserted value is unique
            }
        }
    } else {
        for (j, elem) in curr.iter().enumerate() {
            if i < next.len() && next[i] == curr[j] {
                i += 1;
            } else {
                assert!(out.insert((j, elem.clone()))); // make sure the inserted value is unique
            }
        }
    }
    (add, out)
}
// outputs the diff in the form (subtracted, added)
fn set_diff<T>(curr: &HashSet<T>, next: &HashSet<T>) -> (HashSet<T>, HashSet<T>)
where
    T: Copy + Eq + std::hash::Hash,
{
    (
        curr.difference(next).copied().collect(),
        next.difference(curr).copied().collect(),
    )
}

fn map_diff<S, T>(curr: &HashMap<S, T>, next: &HashMap<S, T>) -> Vec<(DiffType, S, T)>
where
    S: Copy + PartialEq + Eq + core::hash::Hash,
    T: Clone + PartialEq,
{
    let mut out = Vec::new();
    // first check if any values were modified
    // then see which keys were added or removed
    for (key, init_val) in curr.iter() {
        let new = next.get(key);
        if let Some(val) = new {
            if val != init_val {
                out.push((DiffType::Modify, *key, val.clone()));
            }
        } else {
            out.push((DiffType::Remove, *key, init_val.clone()))
        }
    }
    let next_keys = next.keys().copied().collect::<HashSet<_>>();
    let curr_keys = curr.keys().copied().collect::<HashSet<_>>();
    let new_keys = next_keys.difference(&curr_keys);
    for key in new_keys {
        let val = next.get(key).unwrap();
        out.push((DiffType::Add, *key, val.clone()));
    }
    out
}

struct State {
    selections: HashSet<Tag>,
    gcode: Parsed,
}

impl State {
    fn build(gcode: &Parsed) -> Self {
        Self {
            selections: HashSet::new(),
            gcode: gcode.clone(),
        }
    }
    fn gcode_diff(&self, gcode: &Parsed) -> Diff {
        let line_diff = vec_diff(&self.gcode.lines, &gcode.lines);
        let vertex_diff = map_diff(&self.gcode.vertices, &gcode.vertices);
        Diff::GCode(line_diff, vertex_diff)
    }
    fn selection_diff(&self, selection: HashSet<Tag>) -> Diff {
        Diff::Selection(set_diff(&self.selections, &selection))
    }
}
#[derive(Clone, Debug)]
pub enum Diff {
    Init,
    GCode(
        (bool, HashSet<(usize, crate::parser::GCodeLine)>),
        Vec<(DiffType, Id, Vertex)>,
    ),
    Selection((HashSet<Tag>, HashSet<Tag>)),
}

impl Diff {
    fn is_some(&self) -> bool {
        match self {
            Diff::GCode((_, set), vec) => !set.is_empty() || !vec.is_empty(),
            Diff::Selection((sub, add)) => !sub.is_empty() || !add.is_empty(),
            Diff::Init => false,
        }
    }
}

#[derive(Resource)]
pub struct History {
    state: State,
    pub diff_log: VecDeque<Diff>,
    pub counter: usize,
    counter_cur: usize,
}

impl History {
    pub fn build(gcode: &Parsed) -> Self {
        Self {
            state: State::build(gcode),
            diff_log: VecDeque::from([Diff::Init]),
            counter: 0,
            counter_cur: 0,
        }
    }
    fn apply_current(&mut self, forward_or_reverse: bool) {
        let diff = &self.diff_log[self.counter_cur];
        match diff {
            Diff::GCode(line_diff, vertex_diff) => {
                let (dir, set) = line_diff;
                for (i, id) in set.iter() {
                    if *dir == forward_or_reverse {
                        self.state.gcode.lines.insert(*i, id.clone());
                    } else {
                        self.state.gcode.lines.remove(*i);
                    }
                }
                for (diff_type, id, vertex) in vertex_diff {
                    let dir = match diff_type {
                        DiffType::Add | DiffType::Modify => forward_or_reverse == true,
                        DiffType::Remove => forward_or_reverse == false,
                    };
                    if dir {
                        self.state.gcode.vertices.insert(*id, vertex.clone());
                    } else {
                        self.state.gcode.vertices.remove(id);
                    }
                }
            }
            Diff::Selection((sub, add)) => {
                if forward_or_reverse {
                    self.state.selections.extend(add);
                    self.state.selections = self
                        .state
                        .selections
                        .iter()
                        .filter_map(|t| if sub.contains(t) { None } else { Some(*t) })
                        .collect::<HashSet<_>>();
                } else {
                    self.state.selections.extend(sub);
                    self.state.selections = self
                        .state
                        .selections
                        .iter()
                        .filter_map(|t| if add.contains(t) { None } else { Some(*t) })
                        .collect::<HashSet<_>>();
                }
            }
            Diff::Init => {}
        }
    }
    fn apply_to_gcode(&self, gcode: &mut Parsed, forward_or_reverse: bool) {
        let diff = &self.diff_log[self.counter_cur];
        if let Diff::GCode(line_diff, vertex_diff) = diff {
            let (dir, set) = line_diff;
            for (i, id) in set.iter() {
                if *dir {
                    gcode.lines.insert(*i, id.clone());
                } else {
                    gcode.lines.remove(*i);
                }
            }
            for (diff_type, id, vertex) in vertex_diff {
                let dir = match diff_type {
                    DiffType::Add | DiffType::Modify => forward_or_reverse == true,
                    DiffType::Remove => forward_or_reverse == false,
                };
                if dir {
                    gcode.vertices.insert(*id, vertex.clone());
                } else {
                    gcode.vertices.remove(id);
                }
            }
        }
    }
}

pub fn update_history_diff_log(
    mut history: ResMut<History>,
    gcode: Res<GCode>,
    selections: Query<(&PickSelection, &Tag)>,
) {
    let gcode_diff = history.state.gcode_diff(&gcode.0);
    let selections = selections
        .iter()
        .filter_map(|(s, t)| if s.is_selected { Some(*t) } else { None })
        .collect();
    let selection_diff = history.state.selection_diff(selections);
    if selection_diff.is_some() {
        println!("{:?}", selection_diff);
    }
    if (gcode_diff.is_some() || selection_diff.is_some()) && history.counter != history.counter_cur
    {
        history.counter = 0;
        history.counter_cur = 0;
        history.diff_log = VecDeque::new();
    }
    if gcode_diff.is_some() {
        history.diff_log.push_front(gcode_diff);
        history.apply_current(true)
    }
    if selection_diff.is_some() {
        history.diff_log.push_front(selection_diff);
        history.apply_current(true);
    }
}

pub fn undo_redo(
    mut history: ResMut<History>,
    mut gcode: ResMut<GCode>,
    mut selections: Query<(&mut PickSelection, &Tag)>,
) {
    while history.counter != history.counter_cur {
        if history.counter_cur > history.counter {
            history.apply_current(true);
            history.apply_to_gcode(&mut gcode.0, true);
            history.counter_cur -= 1;
        } else {
            history.apply_current(false);
            history.apply_to_gcode(&mut gcode.0, false);
            history.counter_cur += 1;
        }
        // apply state selections to selection query
        for (mut selection, tag) in selections.iter_mut() {
            selection.is_selected = history.state.selections.contains(tag);
        }
    }
}
