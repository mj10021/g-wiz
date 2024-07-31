use super::CommandEvent;

use bevy::{prelude::*, utils::hashbrown::Equivalent};

use std::fmt::{Debug, Formatter};

#[derive(Resource)]
pub struct Console {
    pub current_command: Option<CommandEvent>,
    current_param: Option<char>,
    pub input: String,
    pub output: String,
}
const INTRO: &str = "Welcome to the console. Type 'help' for a list of commands.\r\n";

pub const HELP: &str = "Commands: \r\n\r\n translate \r\n rotate \r\n scale \r\n subdivide \r\n draw \r\n filter \r\n map \r\n";

impl Default for Console {
    fn default() -> Self {
        Self {
            current_command: None,
            current_param: None,
            input: String::new(),
            output: String::from(INTRO),
        }
    }
}
impl Console {
    pub fn send(&mut self, writer: &mut EventWriter<CommandEvent>) {
        if let Some(command) = self.current_command.take() {
            writer.send(command);
        }
    }
    pub fn read(&mut self, input: &str) {
        println!("asdf");
        if self.current_command.is_none() {
            match CommandEvent::build(input) {
                Ok(c) => {
                    println!("asdf1");
                    self.current_command = Some(c.clone());
                }
                Err(e) => {
                    if e.contains("help") && e.len() == 4 {
                        self.output += HELP;
                        return;
                    }
                    println!("asdf2");
                    self.output += &format!("Unknown command: {}\r\n", e);
                }
            }
        }
        println!("asdf3");
    }
    pub fn read_param(&mut self, param: &str) -> Result<(), String> {
        if let Some(command) = &mut self.current_command {
            let command = command.inner_mut();
            if let Some(p) = self.current_param {
                command.set_param(&p, param)
            } else if let Some(c) = param.chars().next() {
                if !command.contains_param(&c) {
                    return Err(format!("invalid parameter character: {}", c));
                }
                self.current_param = Some(c);
                Ok(())
            } else {
                Err(String::from("invalid parameter"))
            }
        } else {
            Err(String::from("no active command"))
        }
    }
}

impl CommandEvent {
    pub fn build(arg: &str) -> Result<Self, String> {
        println!("{}",arg);
        let out = match arg {
            "translate" => Ok(CommandEvent::Translate(Translate::default())),
            "rotate" => Ok(Self::Rotate(Rotate::default())),
            "scale" => Ok(Self::Scale(Scale::default())),
            "subdivide" => Ok(Self::Subdivide(Subdivide::default())),
            // "draw" => Ok(Self::Draw(Draw::default())),
            // "filter" => Ok(Self::Filter(Filter::default())),
            // "map" => Ok(Self::Map(Map::default())),
            "help" => Err(arg.to_string()),
            _ => Err(arg.to_string()),
        };
        println!("Asdfasdfasdf");
        out
    }
}
pub trait Param {
    fn set_param(&mut self, param: &char, value: &str) -> Result<(), String>;
    fn contains_param(&self, param: &char) -> bool;
}

#[derive(Clone)]
pub struct Translate {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub e: Option<f32>,
    pub f: Option<f32>,
    pub preserve_flow: bool,
    params: [char; 5],
}
impl Default for Translate {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            z: None,
            e: None,
            f: None,
            preserve_flow: false,
            params: ['x', 'y', 'z', 'e', 'f'],
        }
    }
}
impl Param for Translate {
    fn set_param(&mut self, param: &char, value: &str) -> Result<(), String> {
        let Ok(value) = value.parse::<f32>() else {
            return Err(value.to_string());
        };
        match param {
            'x' => self.x = Some(value),
            'y' => self.y = Some(value),
            'z' => self.z = Some(value),
            'e' => self.e = Some(value),
            'f' => self.f = Some(value),
            _ => return Err(format!("Unknown parameter: {}", param)),
        }
        Ok(())
    }
    fn contains_param(&self, param: &char) -> bool {
        self.params.contains(param)
    }
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
#[derive(Clone)]
pub struct Rotate {
    pub rho: Option<f32>,
    pub theta: Option<f32>,
    pub phi: Option<f32>,
    params: [char; 3],
}
impl Default for Rotate {
    fn default() -> Self {
        Self {
            rho: None,
            theta: None,
            phi: None,
            params: ['r', 't', 'p'],
        }
    }
}
impl Param for Rotate {
    fn set_param(&mut self, param: &char, value: &str) -> Result<(), String> {
        let Ok(value) = value.parse::<f32>() else {
            return Err(format!("{}", value));
        };
        match param {
            'r' => self.rho = Some(value),
            't' => self.theta = Some(value),
            'p' => self.phi = Some(value),
            _ => return Err(format!("Unknown parameter: {}", param)),
        }
        Ok(())
    }
    fn contains_param(&self, param: &char) -> bool {
        self.params.contains(param)
    }
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
#[derive(Clone)]
pub struct Scale {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    params: [char; 3],
}
impl Default for Scale {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            z: None,
            params: ['x', 'y', 'z'],
        }
    }
}
impl Param for Scale {
    fn set_param(&mut self, param: &char, value: &str) -> Result<(), String> {
        let Ok(value) = value.parse::<f32>() else {
            return Err(value.to_string());
        };
        match param {
            'x' => self.x = Some(value),
            'y' => self.y = Some(value),
            'z' => self.z = Some(value),
            _ => return Err(format!("Unknown parameter: {}", param)),
        }
        Ok(())
    }
    fn contains_param(&self, param: &char) -> bool {
        self.params.contains(param)
    }
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

#[derive(Clone)]
pub struct Subdivide {
    pub count_or_dist: bool,
    pub n: f32,
    params: [char; 3],
}
impl Default for Subdivide {
    fn default() -> Self {
        Self {
            count_or_dist: true,
            n: 1.0,
            params: ['n', 'c', 'd'],
        }
    }
}
impl Param for Subdivide {
    fn set_param(&mut self, param: &char, value: &str) -> Result<(), String> {
        let Ok(value) = value.parse::<f32>() else {
            return Err(format!("{}", value));
        };
        if value > 0.0 {
            self.n = value;
        }
        match param {
            'n' => {}
            'c' => self.count_or_dist = true,
            'd' => self.count_or_dist = false,
            _ => return Err(format!("Unknown parameter: {}", param)),
        }
        Ok(())
    }
    fn contains_param(&self, param: &char) -> bool {
        self.params.contains(param)
    }
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
// #[derive(Clone)]
// pub struct Draw {
//     pub before_or_after: bool,
//     pub start: Option<Pos>,
//     pub end: Option<Pos>,
//     params: [char; 3],
// }
// impl Default for Draw {
//     fn default() -> Self {
//         Self {
//             params: ['b', 's', 'e'],
//         }
//     }
// }
// impl Param for Draw {
//     fn set_param(&mut self, param: &char, value: String) -> Result<(), String> {
//         let Ok(value) = value.parse::<f32>() else {return Err(value)};
//         match param {
//             'b' => self.before_or_after = value > 0.0,
//             's' => self.start = Some(Pos::new(value, 0.0, 0.0)),
//             'e' => self.end = Some(Pos::new(value, 0.0, 0.0)),
//             _ => return Err(format!("Unknown parameter: {}", param)),
//         }
//         Ok(())
//     }
// }
// impl Debug for Draw {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(
//             f,
//             "Draw: <b>efore or <a>fter: {:?}, <s>tart: {:?}, <e>nd: {:?} }}",
//             self.before_or_after, self.start, self.end
//         )
//     }
// }
// #[derive(Clone)]
// pub struct Filter {
//     filter: String,
//     params: [char; 1],
// }
// impl Default for Filter {
//     fn default() -> Self {
//         Self {
//             params: ['f'],
//             
//         }
//     }
// }
// impl Debug for Filter {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "Filter: <f>ilter: {:?} }}", self.filter)
//     }
// }
// #[derive(Clone)]
// pub struct Map {
//     map: String,
//     params: [char; 1],
// }
// impl Default for Map {
//     fn default() -> Self {
//         Self {
//             params: ['m'],
//             ..default()
//         }
//     }
// }
// impl Param for Map {
//     fn set_param(&mut self, param: &char, value: f32) -> Result<(), &str> {
//         match param {
//             'm' => self.map = value.to_string(),
//             _ => return Err(param)
//         }
//         Ok(())
//     }
// }
// impl Debug for Map {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "Map: <m>ap: {:?} }}", self.map)
//     }
// }
//
