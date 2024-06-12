use super::parse::*;
pub trait Emit {
    fn emit(&self, parsed: &Parsed, debug: bool) -> String;
}
impl Emit for Instruction {
    fn emit(&self, _parsed: &Parsed, debug: bool) -> String {
        let Instruction {
            first_word: Word(letter, num, string),
            params,
        } = self;
        if let Some(string) = string {
            return string.clone() + "\n";
        }
        let mut out = format!("{}{}", letter, num.round() as i32);
        if let Some(params) = params {
            for Word(letter, val, _) in params {
                out += &format!(" {}{}", letter, val);
            }
        }
        if debug {
            out += &format!("; {:?}\n", self);
        }
        out + "\n"
    }
}

impl Emit for Pos {
    fn emit(&self, _parsed: &Parsed, debug: bool) -> String {
        if debug {
            return format!(
                "X{} Y{} Z{} E{} F{}; {:?}\n",
                self.x, self.y, self.z, self.e, self.f, self
            );
        }
        assert!(self.x.is_finite() && !self.x.is_nan());
        assert!(self.y.is_finite() && !self.y.is_nan());
        assert!(self.z.is_finite() && !self.z.is_nan());
        assert!(self.e.is_finite() && !self.e.is_nan());
        assert!(self.f.is_finite() && !self.f.is_nan());

        format!(
            "X{} Y{} Z{} E{} F{}\n",
            self.x, self.y, self.z, self.e, self.f
        )
    }
}
impl Emit for Vertex {
    fn emit(&self, parsed: &Parsed, debug: bool) -> String {
        if self.to == Pos::home() && self.prev.is_none() {
            return "G28\n".to_string();
        }
        let from = self.get_from(parsed);
        let mut out = String::from("G1 ");
        if from.x != self.to.x {
            assert!(self.to.x.is_finite() && !self.to.x.is_nan());
            out += &format!("X{} ", self.to.x);
        }
        if from.y != self.to.y {
            assert!(self.to.y.is_finite() && !self.to.y.is_nan());
            out += &format!("Y{} ", self.to.y);
        }
        if from.z != self.to.z {
            assert!(self.to.z.is_finite() && !self.to.z.is_nan());
            out += &format!("Z{} ", self.to.z);
        }
        if self.to.e != 0.0 {
            assert!(self.to.e.is_finite() && !self.to.e.is_nan());
            out += &format!("E{} ", self.to.e);
        }
        if from.f != self.to.f {
            assert!(self.to.f.is_finite() && !self.to.f.is_nan());
            out += &format!("F{} ", self.to.f);
        }
        out += "\n";
        if debug {
            out += &format!("; {:?}\n; {:?}\n; {:?} \n", self.label, from, self.to);
        }
        out
    }
}
impl Emit for Parsed {
    fn emit(&self, _parsed: &Parsed, debug: bool) -> String {
        let mut out = String::new();

        if self.rel_xyz {
            out += "G91\n";
        } else {
            out += "G90\n";
        }
        if self.rel_e {
            out += "M83\n";
        } else {
            out += "M82\n";
        }

        for line in &self.lines {
            if let Some(v) = self.vertices.get(line) {
                out += &v.emit(self, debug);
            } else {
                out += &self.instructions.get(line).unwrap().emit(self, debug);
            }
            //out += "\n";
        }
        out
    }
}

#[test]
fn debug() {
    use std::io::prelude::*;
    use std::fs::File;
    let gcode = Parsed::build("test.gcode", false).expect("");
    let gcode = gcode.emit(&gcode, true);
    let mut f = File::create("test_debug_output.gcode").expect("failed to create file");
    let _ = f.write_all(&gcode.as_bytes());
}