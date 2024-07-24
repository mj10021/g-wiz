pub mod emit;
mod file_reader;
mod transform;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(u32);
impl Id {
    fn get(&mut self) -> Self {
        let out = self.0;
        self.0 += 1;
        Id(out)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Word(pub char, pub f32, pub Option<String>);

#[derive(Clone, Debug, PartialEq)]
pub struct Instruction {
    pub first_word: Word,
    pub params: Option<Vec<Word>>,
}

impl Instruction {
    fn build(mut line: Vec<Word>) -> Instruction {
        let first_word = line.pop().unwrap();
        line.reverse();
        if line.is_empty() {
            return Instruction {
                first_word,
                params: None,
            };
        }
        Instruction {
            first_word,
            params: Some(line),
        }
    }
    pub fn insert_temp_retraction(gcode: &mut Parsed) -> Id {
        let id = gcode.id_counter.get();
        let ins = Instruction {
            first_word: Word('X', f32::NEG_INFINITY, Some(String::from("; retraction"))),
            params: None,
        };
        assert!(gcode.instructions.insert(id, ins).is_none());
        id
    }
    pub fn insert_temp_deretraction(gcode: &mut Parsed) -> Id {
        let id = gcode.id_counter.get();
        let ins = Instruction {
            first_word: Word('X', f32::NEG_INFINITY, Some(String::from("; deretraction"))),
            params: None,
        };
        assert!(gcode.instructions.insert(id, ins).is_none());
        id
    }
}

// intermediary struct for parsing line into vertex
// exists because all of the params are optional
#[derive(Clone, Debug, PartialEq)]
pub struct G1 {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
}

impl G1 {
    fn build(params: Vec<Word>) -> G1 {
        let mut x = None;
        let mut y = None;
        let mut z = None;
        let mut e = None;
        let mut f = None;
        for param in params {
            match param.0 {
                'X' => x = Some(param.1),
                'Y' => y = Some(param.1),
                'Z' => z = Some(param.1),
                'E' => e = Some(param.1),
                'F' => f = Some(param.1),
                _ => (),
            }
        }
        G1 { x, y, z, e, f }
    }
}
// state tracking struct for vertices
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos {
    // abs x, y, z and rel e
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub e: f32,
    pub f: f32,
}

impl Into<Vec3> for Pos {
    fn into(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
impl Pos {
    pub fn home() -> Pos {
        Pos {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            e: 0.0,
            f: f32::NEG_INFINITY, // this will not emit if a feedrate is never set
        }
    }
    pub fn build(prev: &Pos, g1: &G1) -> Pos {
        if pre_home(*prev) {
            panic!("g1 move from unhomed state")
        }
        Pos {
            x: g1.x.unwrap_or(prev.x),
            y: g1.y.unwrap_or(prev.y),
            z: g1.z.unwrap_or(prev.z),
            e: g1.e.unwrap_or(0.0),
            f: g1.f.unwrap_or(prev.f),
        }
    }
    pub fn dist(&self, p: &Pos) -> f32 {
        ((self.x - p.x).powf(2.0) + (self.y - p.y).powf(2.0) + (self.z - p.z).powf(2.0)).sqrt()
    }
}
fn pre_home(p: Pos) -> bool {
    if p.x == f32::NEG_INFINITY
        || p.y == f32::NEG_INFINITY
        || p.z == f32::NEG_INFINITY
        || p.e == f32::NEG_INFINITY
    {
        return true;
    }
    false
}
#[derive(Clone, Copy, PartialEq)]
pub struct Vertex {
    pub id: Id,
    pub count: u32,
    pub label: Label,
    // this is the id of the previous extrusion move
    pub prev: Option<Id>,
    // this is the id of the next extrusion move
    pub next: Option<Id>,
    pub to: Pos,
}
impl std::fmt::Debug for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vertex")
            .field("label", &self.label)
            .field("to", &self.to)
            .finish()
    }
}

impl Vertex {
    fn build(parsed: &mut Parsed, prev: &Id, g1: G1) -> Vertex {
        let id = parsed.id_counter.get();
        let p = parsed.vertices.get_mut(prev).unwrap();
        let mut vrtx = Vertex {
            id,
            count: p.count + 1,
            label: Label::Uninitialized,
            to: Pos::build(&p.to, &g1),
            prev: Some(*prev),
            next: p.next,
        };
        p.next = Some(id);
        vrtx.label(parsed);
        vrtx
    }
    pub fn get_from(&self, parsed: &Parsed) -> Pos {
        if let Some(prev) = self.prev {
            parsed.vertices.get(&prev).unwrap().to
        } else {
            Pos::home()
        }
    }
    fn label(&mut self, parsed: &Parsed) {
        let from = self.get_from(parsed);
        let dx = self.to.x - from.x;
        let dy = self.to.y - from.y;
        let dz = self.to.z - from.z;
        let de = self.to.e;
        self.label = {
            if self.to.x < 5.0 || self.to.y < 5.0 {
                Label::PrePrintMove
            } else if de > 0.0 {
                if dx.abs() + dy.abs() > 0.0 - f32::EPSILON {
                    if dz.abs() > f32::EPSILON {
                        Label::NonPlanarExtrusion
                    } else {
                        Label::PlanarExtrustion
                    }
                } else {
                    Label::DeRetraction
                }
            } else if dz.abs() > f32::EPSILON {
                if dz < 0.0 {
                    Label::LowerZ
                } else {
                    Label::LiftZ
                }
            } else if de.abs() > f32::EPSILON {
                if dx.abs() + dy.abs() > f32::EPSILON {
                    Label::Wipe
                } else {
                    Label::Retraction
                }
            } else if dx.abs() + dy.abs() > f32::EPSILON {
                Label::TravelMove
            } else if from.f != self.to.f {
                Label::FeedrateChangeOnly
            } else {
                Label::MysteryMove
            }
        };
    }
    pub fn extrusion_move(&self) -> bool {
        self.label == Label::PlanarExtrustion || self.label == Label::NonPlanarExtrusion
    }
    pub fn change_move(&self) -> bool {
        self.label == Label::LiftZ || self.label == Label::Wipe || self.label == Label::Retraction
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub id: Id,
    lines: Vec<Id>,
    layer: f32,
}

impl Shape {
    pub fn _len(&self, gcode: &Parsed) -> f32 {
        let mut out = 0.0;
        for line in &self.lines {
            if gcode.vertices.contains_key(line) {
                out += gcode.dist_from_prev(line);
            }
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub lines: Vec<Id>, // keep track of line order
    pub vertices: HashMap<Id, Vertex>,
    pub instructions: HashMap<Id, Instruction>,
    pub shapes: Vec<Shape>,
    pub rel_xyz: bool,
    pub rel_e: bool,
    id_counter: Id,
}
impl Parsed {
    pub fn build(path: &str, testing: bool) -> Result<Parsed, Box<dyn std::error::Error>> {
        let mut parsed = Parsed {
            lines: Vec::new(),
            vertices: HashMap::new(),
            instructions: HashMap::new(),
            shapes: Vec::new(),
            rel_xyz: false,
            rel_e: true,
            id_counter: Id(0),
        };
        let lines = {
            if !testing {
                file_reader::parse_file(path)?
            } else {
                file_reader::parse_str(path)
            }
        };
        assert!(!lines.is_empty());
        // previous vertex id
        let mut prev: Option<Id> = None;
        for line in lines {
            // parse the line into a vec of Word(char, f32, Option<String>)
            let mut line = file_reader::split_line(&line);
            if line.is_empty() {
                continue;
            }
            // reverse the vec to be able to pop from the first commands
            line.reverse();
            // match the first word from the line
            let front = line.pop();
            let Word(letter, number, params) = front.unwrap();
            // lines have already been checked for non integer word numbers
            let num = number.round() as i32;
            match (letter, num) {
                ('G', 28) => {
                    // if the homing node points to a previous extrusion move node, something is wrong
                    assert!(prev.is_none(), "homing from previously homed state");
                    let id = parsed.id_counter.get();
                    let vrtx = Vertex {
                        id,
                        count: 0,
                        label: Label::Home,
                        to: Pos::home(),
                        prev: None,
                        next: None,
                    };
                    assert!(parsed.vertices.insert(id, vrtx).is_none());
                    prev = Some(id);
                    parsed.lines.push(id);
                }
                ('G', 1) => {
                    // if prev is None, it means no homing command has been read
                    let p = prev.expect("g1 move from unhomed state");
                    let g1 = G1::build(line);
                    let vrtx = Vertex::build(&mut parsed, &p, g1);
                    parsed.lines.push(vrtx.id);
                    prev = Some(vrtx.id);
                    assert!(parsed.vertices.insert(vrtx.id, vrtx).is_none());
                }
                ('G', 90) => {
                    parsed.rel_xyz = false;
                }
                ('G', 91) => {
                    parsed.rel_xyz = true;
                }
                ('M', 82) => {
                    parsed.rel_e = false;
                }
                ('M', 83) => {
                    parsed.rel_e = true;
                }
                _ => {
                    let word = Word(letter, number, params);
                    line.push(word);
                    let id = parsed.id_counter.get();
                    let ins = Instruction::build(line);
                    parsed.lines.push(id);
                    assert!(parsed.instructions.insert(id, ins).is_none());
                }
            }
        }
        parsed.assign_shapes();
        Ok(parsed)
    }

    pub fn assign_shapes(&mut self) {
        let mut out = Vec::new();
        let mut temp_shape = Vec::new();
        let mut layer = -1.0;
        for line in &self.lines {
            if let Some(vertex) = self.vertices.get(line) {
                if vertex.extrusion_move() {
                    layer = vertex.to.z;
                }
                if vertex.change_move() {
                    let shape = Shape {
                        id: self.id_counter.get(),
                        lines: temp_shape,
                        layer,
                    };
                    out.push(shape);
                    temp_shape = Vec::new();
                    layer = -1.0;
                } else {
                    temp_shape.push(*line);
                }
            } else {
                temp_shape.push(*line);
            }
        }
        if !temp_shape.is_empty() {
            let shape = Shape {
                id: self.id_counter.get(),
                lines: temp_shape,
                layer,
            };
            out.push(shape);
        }
        self.shapes = out;
    }
    pub fn get_centroid(&self, vertices: &HashSet<Id>) -> Vec3 {
        let (mut x, mut y, mut z, mut count) = (0.0, 0.0, 0.0, 0.0);
        for vertex in vertices {
            count += 1.0;
            let v = self.vertices.get(vertex).unwrap();
            x += v.to.x;
            y += v.to.y;
            z += v.to.z;
        }
        let mut out = Vec3 { x, y, z };
        out /= count;
        out
    }
    fn dist_from_prev(&self, id: &Id) -> f32 {
        let v = self.vertices.get(id).expect("vertex not found in map");
        let p = self
            .vertices
            .get(&v.prev.unwrap())
            .expect("dist from vertex with no prev");
        p.to.dist(&v.to)
    }

    pub fn hole_delete(&mut self, lines_to_delete: &mut HashSet<Id>) {
        for (id, v) in self.vertices.iter_mut() {
            if lines_to_delete.contains(id) {
                v.to.e = 0.0;
            }
        }
    }
    pub fn merge_delete(&mut self, lines_to_delete: &mut HashSet<Id>) {
        let mut temp = Vec::new();

        for line in &self.lines {
            if lines_to_delete.is_empty() {
                break;
            }
            if lines_to_delete.contains(line) {
                lines_to_delete.remove(line);
                //  keep track of the prev node of the first vertex deleted in a block of verteces
                let (_, vertex) = self
                    .vertices
                    .remove_entry(line)
                    .expect("removing non-existent vertex");
                if let Some(n) = vertex.next {
                    let n = self.vertices.get_mut(&n).unwrap();
                    n.prev = vertex.prev;
                }
                if let Some(p) = vertex.prev {
                    let p = self.vertices.get_mut(&p).unwrap();
                    p.next = vertex.next;
                }
            } else {
                temp.push(line);
            }
        }
    }

    fn insert_lines_before(&mut self, mut lines: Vec<Id>, id: &Id) {
        let mut i = 0;
        for line in &self.lines {
            if line == id {
                break;
            }
            i += 1;
        }
        while let Some(line) = lines.pop() {
            self.lines.insert(i, line);
        }
    }
    fn subdivide_vertex(&mut self, id: &Id, count: u32) {
        // FIXME: THIS IS DELETING MOVES
        if count < 1 {
            return;
        }
        // this is assuming relative e
        let v = self.vertices.get(id).unwrap();
        // don't subdivide moves with no extrustion
        if v.label != Label::PlanarExtrustion && v.label != Label::NonPlanarExtrusion {
            return;
        }
        let (xi, yi, zi) = {
            if v.prev.is_none() {
                (0.0, 0.0, 0.0)
            } else {
                let prev = self.vertices.get(&v.prev.unwrap()).unwrap();
                (prev.to.x, prev.to.y, prev.to.z)
            }
        };
        let (xf, yf, zf, ef, f) = (v.to.x, v.to.y, v.to.z, v.to.e, v.to.f);
        let countf = count as f32;
        let (step_x, step_y, step_z) = ((xf - xi) / countf, (yf - yi) / countf, (zf - zi) / countf);
        let mut prev = v.prev;
        let mut vec = Vec::new();
        let mut new_ids = Vec::new();
        for i in 1..count {
            let i = i as f32;
            let mut new = Vertex {
                id: self.id_counter.get(),
                count: 0, // this then needs to be counted and set
                label: Label::Uninitialized,
                prev,
                to: Pos {
                    x: xi + (step_x * i),
                    y: yi + (step_y * i),
                    z: zi + (step_z * i),
                    e: ef / countf,
                    f,
                },
                next: None, // this gets set as part of set_counts
            };
            new.label(self);
            self.vertices.insert(new.id, new);
            prev = Some(new.id);
            new_ids.push(new.id);
            vec.push(new);
        }
        for id in &new_ids {
            prev = Some(*id);
        }
        self.insert_lines_before(new_ids, id);
        let v = self.vertices.get_mut(id).unwrap();
        v.to.e = ef / countf;
        v.prev = prev;
    }
    pub fn subdivide_vertices(&mut self, vertices: HashSet<Id>, count: u32) {
        for id in vertices {
            self.subdivide_vertex(&id, count);
        }
        self.set_counts();
    }
    // FIXME: add ui for this
    pub fn subdivide_all(&mut self, max_dist: f32) {
        let vertices = self.vertices.clone();
        for id in vertices.keys() {
            if self.vertices.contains_key(id) {
                let dist = self.dist_from_prev(id);
                let count = (dist / max_dist).round() as u32;
                self.subdivide_vertex(id, count);
            }
        }
    }

    pub fn get_shape(&self, vertex: &Id) -> Vec<Id> {
        for shape in self.shapes.iter() {
            if shape.lines.contains(vertex) {
                return shape.lines.clone();
            }
        }
        Vec::new()
    }
    pub fn get_same_z(&self, vertex: &Id) -> Vec<Id> {
        let mut out = Vec::new();
        let z = self.vertices.get(vertex).unwrap().to.z;
        for (_, vertex) in self.vertices.iter() {
            if (vertex.to.z - z).abs() < f32::EPSILON {
                out.push(vertex.id);
            }
        }
        out
    }
    pub fn write_to_file(&self, path: &str) -> Result<(), std::io::Error> {
        use std::fs::File;
        let out = self.emit(self, false);
        let mut f = File::create(path)?;
        f.write_all(out.as_bytes())?;
        println!("save successful");
        Ok(())
    }
    fn set_counts(&mut self) {
        let mut count = 0;
        let mut next = None;
        for line in &self.lines {
            if let Some(v) = self.vertices.get_mut(line) {
                v.count = count;
                v.next = next;
                next = Some(v.id);
                count += 1;
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Label {
    Uninitialized,
    Home,
    PrePrintMove,
    TravelMove,
    PlanarExtrustion,
    NonPlanarExtrusion,
    LiftZ,
    LowerZ,
    MysteryMove,
    Retraction,
    DeRetraction,
    Wipe,
    FeedrateChangeOnly,
}

#[cfg(test)]
#[test]
fn tran_test() {
    let test = "G28\ng1x1e1\ng1x2e1\ng1x3e1\n";
    let mut gcode = read(test, true).expect("failed to parse");
    for line in gcode.lines.clone() {
        if gcode.vertices.contains_key(&line) {
            gcode.translate(&line, &Vec3::new(0.0, 1.0, 0.0));
        }
    }
}

#[test]
fn map_test() {
    let mut test = read("test.gcode", false).expect("failed to parse");
    test.subdivide_all(1.0);
}

#[test]
#[should_panic]
fn no_home_test() {
    let input = "G1 X1 Y1 Z1 E1\n";
    let _ = read(input, true).expect("failed to parse");
}
#[test]
#[should_panic]
fn double_home() {
    let _ = read("G28\nG28\nG1 x1\ng1y1\ng1e2.222\ng1z1\n", true).expect("failed to parse");
}

pub fn read(path: &str, raw_str: bool) -> Result<Parsed, Box<dyn std::error::Error>> {
    Parsed::build(path, raw_str)
}

fn _vertex_filter(gcode: &Parsed, f: fn(&Vertex) -> bool) -> HashSet<Id> {
    let mut out = HashSet::new();
    for line in &gcode.lines {
        if let Some(v) = gcode.vertices.get(line) {
            if f(v) {
                out.insert(v.id);
            }
        }
    }
    out
}

// fn insert_before(feature)
// fn modify(feature)
// fn replace_with(feature, gcode_sequence)
// fn insert_after(feature)

#[cfg(test)]
use std::fs::File;
use std::io::Write;

use bevy::math::Vec3;
use emit::Emit;
#[test]
fn import_emit_reemit() {
    use emit::Emit;
    use std::io::prelude::*;
    let f = "../print_analyzer/test.gcode";
    let p_init = read(f, false).expect("failed to parse gcode");
    let init = p_init.emit(&p_init, false);
    let mut f = File::create("test_output.gcode").expect("failed to create file");
    let _ = f.write_all(init.as_bytes());
    let snd = read("test_output.gcode", false).expect("asdf");
    let snd = snd.emit(&snd, false);
    let snd = read(&snd, true).expect("failed to parse reemitted file");
    let mut f = File::create("test_output2.gcode").expect("failed to create file");
    let _ = f.write_all(snd.emit(&snd, false).as_bytes());
    // assert_eq!(p_init, snd);
}
#[test]
fn specific_random_gcode_issue() {
    use emit::Emit;
    use std::io::prelude::*;
    let gcode = "G28
    G1 X179 Y-2 F2400 
    G1 Z3 F720 
    G1 X170 F1000 
    G1 Z0.2 F720 
    ; END LAYER CHANGE
    ; START SHAPE
    G1 X110 E8 F900 
    G1 X40 E10 F700 
    G92 E0
    M221 S95
    G21
    G90
    M83
    M900 K0.06
    M107
    G92 E0
    M73 P1 R11
    ; END SHAPE
    M73 P2 R11
    ; START SHAPE CHANGE
    G1 F720 
    G1 Z0.3 
    G1 Z0.5 
    G1 X78.662 Y77.959 F9000 
    G1 Z0.3 F720 
    G1 E3 F1200
    G1 X78.663 Y78 E3.000 F1200
    G1 X87 Y83 E13";
    let gcode = read(gcode, true).expect("asf");
    let mut f = File::create("asdf_test.gcode").expect("failed to create file");
    let _ = f.write_all(gcode.emit(&gcode, false).as_bytes());
}
