use crate::{Diff, GCodeModel, Id};
use g_win::{Command, G1};



#[derive(Resource, Diff)]
pub struct State {
    pub selections: HashSet<Tag>,
    pub geometry: GeometryMap,
    pub gcode: Vec<String>,
}

impl State {
    fn build(geometry: GeometryMap, gcode: GCodeModel) -> Self {
        Self {
            selections: HashSet::new(),
            geometry,
            gcode: gcode.emit(false).split('\n').map(|s| s.to_string()).collect(),
        }
    }
}



#[derive(Clone, Diff)]
pub struct GeometryMap {
    pub vertices: VertexMap,
    pub shapes: ShapeMap,
    pub layers: LayerMap,
}

#[derive(Clone, Default, Debug, Diff, PartialEq)]
pub struct VertexMap(pub HashMap<Id, Vertex>);
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Shape {
    id: Id,
    vertices: Vec<Id>,
}
#[derive(Default, Debug, Clone, PartialEq)]
pub struct ShapeMap(pub HashMap<Id, Shape>);
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Layer {
    id: Id,
    shapes: Vec<Id>,
}
#[derive(Default, Debug, Clone, PartialEq)]
pub struct LayerMap(pub HashMap<Id, Layer>);

impl VertexMap {
    pub fn build(gcode: &GCodeModel) -> Self {
        let mut out = Self::default();
        let mut prev = None;
        for line in &gcode.lines {
            if let Command::G1(g1) = &line.command {
                let vertex = Vertex::build(g1, prev, &out);
                out.0.insert(line.id, vertex.clone());
                prev = Some(line.id);
            }
        }
        out
    }
}

use std::{collections::HashMap, ops::Index};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    #[default]
    Uninitialized,
    Extrusion,
    Retraction,
    Wipe,
    DeRetraction,
    LiftZ,
    LowerZ,
    Travel,
    FeedrateChange,
}

pub trait Pos5<P>
where
    P: Index<usize, Output = f32>,
{
    fn dist_xyz(&self, other: P) -> f32;
    fn flow(&self, other: P) -> f32;
}

impl<P> Pos5<P> for [f32; 5]
where
    P: Index<usize, Output = f32>,
{
    fn dist_xyz(&self, other: P) -> f32 {
        let dx: f32 = self[0] - other[0];
        let dy: f32 = self[1] - other[1];
        let dz: f32 = self[2] - other[1];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    fn flow(&self, other: P) -> f32 {
        // self e / dist from prev
        self[3] / self.dist_xyz(other)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// [x, y, z, e, f]
    pub position: [f32; 5],
    pub prev: Option<Id>,
    pub label: Label,
}

impl Vertex {
    pub fn build(g1: &G1, prev: Option<Id>, vertices: &VertexMap) -> Self {
        let init = if let Some(prev) = prev {
            vertices.0[&prev].position
        } else {
            [0.0, 0.0, 0.0, 0.0, 0.0]
        };
        let mut curr = [0.0; 5];
        let G1 { x, y, z, e, f } = g1;
        let vals = [x, y, z, e, f];
        for i in 0..5 {
            if let Some(val) = vals[i] {
                curr[i] = val.as_str().parse::<f32>().unwrap_or(init[i]);
            } else {
                curr[i] = init[i];
            }
        }
        let mut out = Self {
            position: curr,
            prev,
            label: Label::Uninitialized,
        };
        out.label = out.label(vertices);
        out
    }
    fn label(&mut self, map: &VertexMap) -> Label {
        let [xi, yi, zi, _, fi] = {
            if let Some(prev) = self.prev {
                map.0[&prev].position
            } else {
                [0.0, 0.0, 0.0, 0.0, 0.0]
            }
        };
        let [xf, yf, zf, e, f] = self.position;
        let dx = xf - xi;
        let dy = yf - yi;
        let dz = zf - zi;
        if e > 0.0 {
            if dx > 0.0 || dy > 0.0 || dz > 0.0 {
                Label::Extrusion
            } else {
                Label::DeRetraction
            }
        } else if e > std::f32::EPSILON {
            // if e == 0.0
            if dz > 0.0 {
                Label::LiftZ
            } else if dz < 0.0 {
                Label::LowerZ
            } else if dx > 0.0 || dy > 0.0 {
                Label::Travel
            } else if f != fi {
                Label::FeedrateChange
            } else {
                panic!("Unreachable code path")
            }
        } else {
            // if e < 0.0
            if dx > 0.0 || dy > 0.0 || dz > 0.0 {
                Label::Wipe
            } else {
                Label::Retraction
            }
        }
    }
    pub fn x(&self) -> f32 {
        self.position[0]
    }
    pub fn y(&self) -> f32 {
        self.position[1]
    }
    pub fn z(&self) -> f32 {
        self.position[2]
    }
    pub fn e(&self) -> f32 {
        self.position[3]
    }
    pub fn f(&self) -> f32 {
        self.position[4]
    }
}
