use bevy::app::App;

use crate::{print_analyzer::{Id, Pos}, GCode};
use std::{collections::HashSet};

enum CommandId {
    Translate,
    Rotate,
    Scale,
    Subdivide,
    Draw,
    Filter,
    Map
}

impl CommandId {
    fn build(id: &str) -> Result<Self, &str> {
        let out = match id {
            "translate" => Self::Translate,
            "rotate" => Self::Rotate,
            "scale" => Self::Scale,
            "subdivide" => Self::Subdivide,
            "draw" => Self::Draw,
            "filter" => Self::Filter,
            "map" => Self::Map,
            _ => return Err(id)
        };
        Ok(out)
    }
}

trait Apply {
    fn apply(&self, gcode: &mut GCode) -> Result<(), &str>;
}
struct Translate {
    selection: HashSet<Id>,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    e: Option<f32>,
    f: Option<f32>,
    preserve_flow: bool,
}

impl Apply for Translate {
    fn apply(&self, gcode: &mut GCode) -> Result<(), &str> {
        for id in &self.selection {
            if let Some(v) = gcode.0.vertices.get_mut(id) {
                v.to.x += self.x.unwrap_or(0.0);
                v.to.y += self.y.unwrap_or(0.0);
                v.to.z += self.z.unwrap_or(0.0);
                if !self.preserve_flow {
                    v.to.e = self.e.unwrap_or(0.0);
                }
                if self.f.is_some() {v.to.f = self.f.unwrap();}
            } else {
                return Err("vertex not found");
            }
        }
        Ok(())
    }
}

struct Rotate {
    selection: HashSet<Id>,
    rho: Option<f32>,
    theta: Option<f32>,
    phi: Option<f32>
}

impl Apply for Rotate {
    fn apply(&self, gcode: &mut GCode) -> Result<(), &str> {
        let origin = gcode.0.get_centroid(&self.selection);
        for id in self.selection.iter() {
            gcode.0.rotate(&id, origin, self.rho.unwrap_or(0.0), self.theta.unwrap_or(0.0), self.phi.unwrap_or(0.0));
        }
        Ok(())
    }
}
 
struct Scale {
    selection: HashSet<Id>,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>
}

impl Apply for Scale {
    fn apply(&self, gcode: &mut GCode) -> Result<(), &str> {
        let origin = gcode.0.get_centroid(&self.selection);
        let (x, y, z) = x.unwrap_or(1.0), y.unwrap_or(1.0), z.unwrap_or(1.0);
        for id in self.selection.iter() {
            let Some(v) = gcode.0.vertices.get_mut(id) else {return Err("vertex not found")};
            v.to.x = origin.x + (v.to.x - origin.x) * x;
            v.to.y = origin.y + (v.to.y - origin.y) * y;
            v.to.z = origin.z + (v.to.z - origin.z) * z;
        }
        Ok(())
    }

}

// struct Subdivide {
//     selection: HashSet<Id>,
//     count_or_dist: bool,
//     n: f32
// }
// 
// impl Apply for Subdivide {
//     fn apply(&self, gcode: &mut GCode) -> Result<(), &str> {
//         if self.selection.is_empty() {
//             gcode.0.subdivide_all(n);
//         } else {
//             for id in self.selection.iter() {
//                 gcode.0.subdivide(&id, self.count_or_dist, self.n);
//             }
//         }
//         Ok(())
//     }
// }

// struct Draw {
//     next_node: Option<Id>,
//     before_or_after: bool,
//     start: Option<Pos>,
//     end: Option<Pos>
// }
// 
// struct Filter {
//     selection: HashSet<Id>,
//     filter: String
// }
// 
// struct Map {
//     selection: HashSet<Id>,
//     map: String
// }
struct Command {
    id: CommandId,
    options: Vec<String>,
    params: Vec<String>,
}

impl Command {
    fn build(args: &str) -> Result<Self, &str> {
        // format commands "$ translate"
        // response like "translate <x> dist: 0.0, <y> dist: 0.0, <z> dist: 0.0, <e>xtrusion: 0.0, <f>eedrate: 0.0, <p>reserve_flow: false"


        let mut args = args.split_whitespace();
        if let Some(command_id) = args.next() {
            let 
        } else {
            return Err("no command entered");
        }
    }
    
}