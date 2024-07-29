use crate::print_analyzer::Pos;
use super::CommandEvent;
use bevy::prelude::*;
use std::fmt::{Debug, Formatter};

#[derive(Resource)]
pub struct Console {
    current_command: Option<CommandEvent>,
    pub input: String,
    pub output: String,
}
const INTRO: &str = "Welcome to the console. Type 'help' for a list of commands.\r\n";

const HELP: &str = "Commands: \r\n\r\n translate \r\n rotate \r\n scale \r\n subdivide \r\n draw \r\n filter \r\n map \r\n";

impl Default for Console {
    fn default() -> Self {
        Self {
            current_command: None,
            input: String::new(),
            output: String::from(INTRO),
        }
    }
}

impl Console {
    pub fn read_command(&mut self, input: &String) {
        let input = CommandEvent::build(input);
        match input {
            Ok(c) => {
                self.current_command = Some(c);
                // console response event here
            }
            Err("help") => {
                self.output += HELP;
            }
            Err(e) => {
                self.output += &format!("Unknown command: {}\r\n", e);
            }
        }
    }
    fn read_params(&mut self) {}
}

impl CommandEvent {
    pub fn build(arg: &str) -> Result<Self, &str> {
        match arg {
            "translate" => Ok(Self::Translate(Translate::default())),
            "rotate" => Ok(Self::Rotate(Rotate::default())),
            "scale" => Ok(Self::Scale(Scale::default())),
            "subdivide" => Ok(Self::Subdivide(Subdivide::default())),
            "draw" => Ok(Self::Draw(Draw::default())),
            "filter" => Ok(Self::Filter(Filter::default())),
            "map" => Ok(Self::Map(Map::default())),
            "help" => Err(arg),
            _ => Err(arg),
        }
    }
}
#[derive(Default)]
pub struct Translate {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
    pub preserve_flow: bool,
}
impl Debug for Translate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Translate: <x>: {:?}, <y>: {:?}, <z>: {:?}, <e>: {:?}, <f>: {:?}, <p>reserve flow: {:?} }}",
            self.x.unwrap_or(0.0),
            self.y.unwrap_or(0.0),
            self.z.unwrap_or(0.0),
            self.e.unwrap_or(0.0),
            self.f.unwrap_or(0.0),
            self.preserve_flow
        )
    }
}
#[derive(Default)]
pub struct Rotate {
    pub rho: Option<f32>,
    pub theta: Option<f32>,
    pub phi: Option<f32>,
}
impl Debug for Rotate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Rotate: <r>ho: {:?}, <t>heta: {:?}, <p>hi: {:?} }}",
            self.rho, self.theta, self.phi
        )
    }
}
#[derive(Default)]
pub struct Scale {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Scale: <x>: {:?}, <y>: {:?}, <z>: {:?} }}",
            self.x.unwrap_or(1.0),
            self.y.unwrap_or(1.0),
            self.z.unwrap_or(1.0)
        )
    }
}

#[derive(Default)]
pub struct Subdivide {
    pub count_or_dist: bool,
    pub n: f32,
}

impl Debug for Subdivide {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Subdivide: <n>: {:?}, divide by <c>ount or segment <d>istance: {:?} }}",
            self.n, self.count_or_dist
        )
    }
}
#[derive(Default)]
pub struct Draw {
    pub before_or_after: bool,
    pub start: Option<Pos>,
    pub end: Option<Pos>,
}

impl Debug for Draw {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Draw: <b>efore or <a>fter: {:?}, <s>tart: {:?}, <e>nd: {:?} }}",
            self.before_or_after, self.start, self.end
        )
    }
}
#[derive(Default)]
pub struct Filter {
    filter: String,
}
impl Debug for Filter {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Filter: <f>ilter: {:?} }}", self.filter)
    }
}
#[derive(Default)]
pub struct Map {
    map: String,
}
impl Debug for Map {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Map: <m>ap: {:?} }}", self.map)
    }
}
