mod emit;
mod parse;
use parse::file_reader::read_line;
pub use parse::{Parsed, Pos, Vertex};
pub use emit::Emit;
use super::Uuid;

use std::collections::HashSet;

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
    pub fn insert_before (&mut self, line: String, pos: &Uuid) {
        let line = read_line(&line);

        todo!();}
}
// fn insert_before(feature)
// fn modify(feature)
// fn replace_with(feature, gcode_sequence)
// fn insert_after(feature)

mod integration_tests {

    #[cfg(test)]
    use std::fs::File;
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
