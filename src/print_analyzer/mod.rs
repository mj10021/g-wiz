pub mod emit;
mod file_reader;
use std::collections::{HashMap, HashSet};
use std::f32::{EPSILON, NEG_INFINITY};
use crate::Uuid;
pub use emit::Emit;

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
        if line.len() < 1 {
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
impl std::ops::Sub for Pos {
    type Output = (f32, f32, f32);
    fn sub(self, rhs: Pos) -> Self::Output {
        (self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl Pos {
    pub fn home() -> Pos {
        Pos {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            e: 0.0,
            f: NEG_INFINITY, // this will not emit if a feedrate is never set
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
    if p.x == NEG_INFINITY || p.y == NEG_INFINITY || p.z == NEG_INFINITY || p.e == NEG_INFINITY {
        return true;
    }
    false
}
#[derive(Clone, PartialEq)]
pub struct Vertex {
    pub id: Uuid,
    pub count: u32,
    pub label: Label,
    // this id of previous extrusion move
    pub prev: Option<Uuid>,
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
    fn build(parsed: &Parsed, prev: Option<Uuid>, g1: G1) -> Vertex {
        let (p, count) = {
            if let Some(prev) = prev.clone() {
                let prev = parsed.vertices.get(&prev).unwrap();
                (prev.to.clone(), prev.count + 1)
            } else {
                (Pos::home(), 0)
            }
        };
        let mut vrtx = Vertex {
            id: Uuid::new_v4(),
            count,
            label: Label::Uninitialized,
            to: Pos::build(&p, &g1),
            prev,
        };
        vrtx.label(parsed);
        vrtx
    }
    pub fn get_from(&self, parsed: &Parsed) -> Pos {
        if let Some(prev) = self.prev.clone() {
            parsed.vertices.get(&prev).unwrap().to.clone()
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
                if dx.abs() + dy.abs() > 0.0 - EPSILON {
                    if dz.abs() > f32::EPSILON {
                        Label::NonPlanarExtrusion
                    } else {
                        Label::PlanarExtrustion
                    }
                } else {
                    Label::DeRetraction
                }
            } else if dz.abs() > EPSILON {
                if dz < 0.0 {
                    Label::LowerZ
                } else {
                    Label::LiftZ
                }
            } else if de.abs() > EPSILON {
                if dx.abs() + dy.abs() > EPSILON {
                    Label::Wipe
                } else {
                    Label::Retraction
                }
            } else if dx.abs() + dy.abs() > EPSILON {
                Label::TravelMove
            } else if from.f != self.to.f {
                Label::FeedrateChangeOnly
            } else {
                Label::MysteryMove
            }
        };
    }
    pub fn is_extrusion(&self) -> bool {
        self.label == Label::PlanarExtrustion || self.label == Label::NonPlanarExtrusion
    }
    pub fn extrusion_move(&self) -> bool {
        self.label == Label::PlanarExtrustion || self.label == Label::NonPlanarExtrusion
    }
    pub fn change_move(&self) -> bool {
        self.label == Label::LiftZ || self.label == Label::Wipe || self.label == Label::Retraction
    }
}

#[derive(Debug, PartialEq)]
pub struct Shape {
    pub id: Uuid,
    lines: Vec<Uuid>,
    layer: f32,
}

impl Shape {
    pub fn len(&self, gcode: &Parsed) -> f32 {
        let mut out = 0.0;
        for line in &self.lines {
            if gcode.vertices.contains_key(line) {
                out += gcode.dist_from_prev(line);
            }
        }
        out
    }
}

#[derive(Debug, PartialEq)]
pub struct Parsed {
    pub lines: Vec<Uuid>,
    pub vertices: HashMap<Uuid, Vertex>,
    pub instructions: HashMap<Uuid, Instruction>,
    pub shapes: Vec<Shape>,
    pub layers: Vec<HashSet<Uuid>>,
    pub rel_xyz: bool,
    pub rel_e: bool,
}
impl Parsed {
    pub fn new() -> Parsed {
        Parsed {
            lines: Vec::new(),
            vertices: HashMap::new(),
            instructions: HashMap::new(),
            shapes: Vec::new(),
            layers: Vec::new(),
            rel_xyz: false,
            rel_e: true,
        }
    }
    pub fn build(path: &str, testing: bool) -> Result<Parsed, Box<dyn std::error::Error>> {
        let mut parsed = Parsed::new();
        let lines = {
            if !testing {
                file_reader::parse_file(path)?
            } else {
                file_reader::parse_str(path)
            }
        };
        assert!(lines.len() > 0);
        // prev holds a raw mut pointer to the to position of the previous vertex
        let mut prev: Option<Uuid> = None;
        for line in lines {
            // parse the line into a vec of words (currently storing the instruction numbers and paramters both as floats)
            let mut line = file_reader::read_line(&line);
            if line.is_empty() {
                continue;
            }
            line.reverse();
            // throw away logical line numbers
            let mut front = line.pop();
            match front.clone() {
                Some(Word('N', _, _)) => {
                    front = line.pop();
                }
                Some(_) => {}
                None => {} //panic!("popping empty line"),
            }
            match front {
                Some(Word(letter, number, params)) => {
                    let num = number.round() as i32;
                    match (letter, num) {
                        ('G', 28) => {
                            // if the homing node points to a previous extrusion move node, something is wrong
                            assert!(prev.is_none(), "homing from previously homed state");
                            let id = Uuid::new_v4();
                            let vrtx = Vertex {
                                id,
                                count: 0,
                                label: Label::Home,
                                to: Pos::home(),
                                prev: None,
                            };
                            parsed.vertices.insert(id.clone(), vrtx);
                            prev = Some(id);
                            parsed.lines.push(id);
                        }
                        ('G', 1) => {
                            // if prev is None, it means no homing command has been read
                            assert!(prev.is_some(), "g1 move from unhomed state");
                            let g1 = G1::build(line);
                            let vrtx = Vertex::build(&parsed, prev, g1);
                            parsed.lines.push(vrtx.id);
                            prev = Some(vrtx.id);
                            parsed.vertices.insert(vrtx.id, vrtx);
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
                            let id = Uuid::new_v4();
                            let ins = Instruction::build(line);
                            parsed.lines.push(id.clone());
                            parsed.instructions.insert(id.clone(), ins);
                        }
                    }
                }
                _ => {
                    panic!("{:?}", front);
                }
            }
        }
        parsed.assign_shapes();
        parsed.assign_layers();
        Ok(parsed)
    }

    fn assign_shapes(&mut self) {
        let mut out = Vec::new();
        let mut temp_shape = Vec::new();
        let mut layer = -1.0;
        for line in &self.lines {
            if let Some(vertex) = self.vertices.get(&line) {
                if vertex.extrusion_move() {
                    layer = vertex.to.z;
                }
                if vertex.change_move() {
                    let shape = Shape {
                        id: Uuid::new_v4(),
                        lines: temp_shape,
                        layer,
                    };
                    out.push(shape);
                    temp_shape = Vec::new();
                    layer = -1.0;
                } else {
                    temp_shape.push(line.clone());
                }
            } else {
                temp_shape.push(line.clone());
            }
        }
        if !temp_shape.is_empty() {
            let shape = Shape {
                id: Uuid::new_v4(),
                lines: temp_shape,
                layer,
            };
            out.push(shape);
        }
        self.shapes = out;
    }

    fn assign_layers(&mut self) {
        assert!(!self.shapes.is_empty(), "building layers with empty shapes");
        let mut curr = self.shapes[0].layer;
        let mut temp = HashSet::new();
        for shape in &self.shapes {
            if (shape.layer - curr).abs() > 0.0 - f32::EPSILON {
                curr = shape.layer;
                self.layers.push(temp);
                temp = HashSet::new();
            }
            temp.insert(shape.id);
        }
    }
    fn dist_from_prev(&self, id: &Uuid) -> f32 {
        let v = self
            .vertices
            .get(id)
            .expect(&format!("vertex with id: {:?} not found in map", id));
        if v.prev.is_none() {
            return 0.0;
        }
        let p = self.vertices.get(&v.prev.unwrap()).unwrap();
        p.to.dist(&v.to)
    }
    // FIXME: need to write a test for this
    pub fn delete_lines(&mut self, lines_to_delete: &mut HashSet<Uuid>) {
        let mut temp = Vec::new();
        let mut last_del: Option<Uuid> = None;
        let mut last_del_prev: Option<Uuid> = None;

        for line in &self.lines {
            if lines_to_delete.is_empty() {
                break;
            }
            if lines_to_delete.contains(line) {
                lines_to_delete.remove(line);
                //  keep track of the prev node of the first vertex deleted in a block of verteces
                let (line, vertex) = self
                    .vertices
                    .remove_entry(line)
                    .expect("removing non-existent vertex");
                if last_del.is_none() {
                    last_del = Some(line);
                    last_del_prev = vertex.prev.clone();
                }
            } else {
                if let Some(vertex) = self.vertices.get_mut(line) {
                    if vertex.prev == last_del {
                        vertex.prev = last_del_prev.clone();
                        last_del = None;
                        last_del_prev = None;
                    }
                }
                temp.push(line.clone());
            }
        }
        self.lines = temp;
    }

    fn _translate(&mut self, id: &Uuid, dx: f32, dy: f32, dz: f32) {
        let v = self.vertices.get(id).unwrap();
        if v.prev.is_none() {
            let v = self.vertices.get_mut(id).unwrap();

            // FIXME: need to adjust flow here
            v.to.x += dx;
            v.to.y += dy;
            v.to.z += dz;
            return;
        }

        let prev = v.prev.unwrap();
        let init_dist = self.dist_from_prev(id);
        let init_flow = self.vertices.get(id).unwrap().to.e;
        let prev_dist = self.dist_from_prev(&prev);
        {
            let pv = self.vertices.get_mut(&prev).unwrap();
            pv.to.x += dx;
            pv.to.y += dy;
            pv.to.z += dz;
        }

        let new_prev_dist = self.dist_from_prev(&prev);

        let prev = self.vertices.get_mut(&prev).unwrap();

        let mut scale = new_prev_dist / prev_dist;
        if scale.is_infinite() || scale.is_nan() {
            scale = 0.0;
        }
        prev.to.e *= scale;

        let new_dist = self.dist_from_prev(id);
        let mut scale = new_dist / init_dist;
        if scale.is_infinite() || scale.is_nan() {
            scale = 0.0;
        }
        let v = self.vertices.get_mut(id).unwrap();
        v.to.e = init_flow * scale;
    }
    fn insert_lines_before(&mut self, mut lines: Vec<Uuid>, id: &Uuid) {
        let mut i = 0;
        for line in &self.lines {
            if line == id {
                break;
            }
            i += 1;
        }
        while lines.len() > 0 {
            self.lines.insert(i, lines.pop().unwrap());
        }
    }
    fn subdivide_vertex(&mut self, id: &Uuid, count: u32) {
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
                id: Uuid::new_v4(),
                count: 0, //FIXME: duh
                label: Label::Uninitialized,
                prev,
                to: Pos {
                    x: xi + (step_x * i),
                    y: yi + (step_y * i),
                    z: zi + (step_z * i),
                    e: ef / countf,
                    f,
                },
            };
            new.label(self);
            self.vertices.insert(new.id, new.clone());
            prev = Some(new.id);
            new_ids.push(new.id);
            vec.push(new);
        }
        // FIXME: this does not seem efficient
        for id in &new_ids {
            prev = Some(id.clone());
        }
        self.insert_lines_before(new_ids, id);
        let v = self.vertices.get_mut(id).unwrap();
        v.to.e = ef / countf;
        v.prev = prev;
    }
    pub fn subdivide_all(&mut self, max_dist: f32) {
        // FIXME: this is probably dumb
        let mut v = Vec::new();
        for key in self.vertices.keys() {
            v.push(key.clone());
        }
        for line in &v {
            let dist = self.dist_from_prev(line);
            let count = (dist / max_dist).round() as u32;
            self.subdivide_vertex(line, count);
        }
    }
    pub fn write_to_file(&self, output_name: &str) -> Result<(), std::io::Error> {
        use std::fs::File;
        use std::io::prelude::*;
        let output = self.emit(&self, false);

        let mut f = File::create(output_name).expect("failed to create file");
        f.write_all(&output.as_bytes())
    }

    pub fn get_shape(&self, vertex: &Uuid) -> Vec<Uuid> {
        for shape in self.shapes.iter() {
            if shape.lines.contains(vertex) {
                return shape.lines.clone();
            }
        }
        Vec::new()
    }
    pub fn get_layer(&self, vertex: &Uuid) -> HashSet<Uuid> {
        for layer in self.layers.iter() {
            if layer.contains(vertex) {
                return layer.clone();
            }
        }
        HashSet::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Label {
    Uninitialized,
    Home,
    _FirstG1,
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
            gcode._translate(&line, 0.0, 1.0, 0.0);
        }
    }
}

#[test]
fn subdivide_all_test() {
    let mut test = read("test.gcode", false).expect("failed to parse");
    test.subdivide_all(1.0);
    assert!(test.write_to_file("subdivide_test.gcode").is_ok());
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

fn _vertex_filter(gcode: &Parsed, f: fn(&Vertex) -> bool) -> HashSet<Uuid> {
    let mut out = HashSet::new();
    for line in &gcode.lines {
        if let Some(v) = gcode.vertices.get(line) {
            if f(v) {
                out.insert(v.id.clone());
            }
        }
    }
    out
}

impl Parsed {
    pub fn insert_before (&mut self, line: &String, pos: &HashSet<Uuid>) {
        let line = file_reader::read_line(line);

        todo!();
    }
}
// fn insert_before(feature)
// fn modify(feature)
// fn replace_with(feature, gcode_sequence)
// fn insert_after(feature)

mod integration_tests {

    #[cfg(test)]
    use std::fs::File;

    use crate::print_analyzer::{read,Emit};
    #[test]
    fn import_emit_reemit() {
        use std::io::prelude::*;
        let f = "test.gcode";
        let p_init = read(f, false).expect("failed to parse gcode");
        let init = p_init.emit(&p_init, false);

        let mut f = File::create("test_output.gcode").expect("failed to create file");
        let _ = f.write_all(&init.as_bytes());
        let snd = read("test_output.gcode", false).expect("asdf");
        let snd = snd.emit(&snd, false);
        let snd = read(&snd, true).expect("failed to parse reemitted file");
        let mut f = File::create("test_output2.gcode").expect("failed to create file");
        let _ = f.write_all(&snd.emit(&snd, false).as_bytes());
        // assert_eq!(p_init, snd);
    }
    #[test]
    fn specific_random_gcode_issue() {
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
        let _ = f.write_all(&gcode.emit(&gcode, false).as_bytes());
    }
}
