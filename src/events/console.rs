use std::fmt::Debug;
use crate::{print_analyzer::{Id, Pos}, GCode, CommandEvent};

struct Console {
    current_command: Option<CommandId>,
    input: String,
    output: String,
}
const intro: &str = "Welcome to the console. Type 'help' for a list of commands.";

const help: &str = "Commands: \r\n\r\n translate \r\n rotate \r\n scale \r\n subdivide \r\n draw \r\n filter \r\n map \r\n";

impl Default for Console {
    fn default() -> Self {
        Self {
            current_command: None,
            input: String::new(),
            output: String::from(intro),
        }
    }
}

enum CommandId {
    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
    Subdivide(Subdivide),
    Draw(Draw),
    Filter(Filter),
    Map(Map)
}

impl CommandId {
    fn build(arg: &str) -> Result<Self, &str> {
        match arg {
            "translate" => Ok(Self::Translate(Translate::default())),
            "rotate" => Ok(Self::Rotate(Rotate::default())),
            "scale" => Ok(Self::Scale(Scale::default())),
            "subdivide" => Ok(Self::Subdivide(Subdivide::default())),
            "draw" => Ok(Self::Draw(Draw::default())),
            "filter" => Ok(Self::Filter(Filter::default())),
            "map" => Ok(Self::Map(Map::default())),
            _ => return Err(arg)
        }
    }
}
    
trait EmitEvent {
    fn emit(&self) -> CommandEvent;
}

#[derive(Default)]
pub struct Translate {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    e: Option<f32>,
    f: Option<f32>,
    preserve_flow: bool,
}
#[derive(Default)]
pub struct Rotate {
    rho: Option<f32>,
    theta: Option<f32>,
    phi: Option<f32>
}
impl Debug for Rotate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Rotate: <r>ho: {:?}, <t>heta: {:?}, <p>hi: {:?} }}", self.rho, self.theta, self.phi);
    }
}
 #[derive(Default)]
pub struct Scale {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>
}
impl EmitEvent for Scale {
    fn emit(&self) -> CommandEvent {
        CommandEvent::Scale {
            x: self.x.unwrap_or(1.0),
            y: self.y.unwrap_or(1.0),
            z: self.z.unwrap_or(1.0)
        }
    }
}
impl Debug for Scale {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Scale: <x>: {:?}, <y>: {:?}, <z>: {:?} }}", self.x, self.y, self.z);
    }
}

#[derive(Default)]
pub struct Subdivide {
    count_or_dist: bool,
    n: f32
}
impl EmitEvent for Subdivide {
    fn emit(&self) -> CommandEvent {
        CommandEvent::Subdivide {
            n: self.n,
            count_or_dist: self.count_or_dist
        }
    }
}
impl Debug for Subdivide {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Subdivide: <n>: {:?}, divide by <c>ount or segment <d>istance: {:?} }}", self.n, self.count_or_dist);
    }
}
#[derive(Default)]
pub struct Draw {
    next_node: Option<Id>,
    before_or_after: bool,
    start: Option<Pos>,
    end: Option<Pos>
}
impl EmitEvent for Draw {
    fn emit(&self) -> CommandEvent {
        CommandEvent::Draw {
            next_node: self.next_node,
            before_or_after: self.before_or_after,
            start: self.start,
            end: self.end
        }
    }
}
impl Debug for Draw {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Draw: <b>efore or <a>fter: {:?}, <s>tart: {:?}, <e>nd: {:?} }}", self.next_node, self.before_or_after, self.start, self.end);
    }
}
#[derive(Default)]
 pub struct Filter {
     selection: HashSet<Id>,
     filter: String
 }
 impl Debug for Filter {
     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
         write!(f, "Filter: <f>ilter: {:?} }}", self.filter);
     }
 }
 #[derive(Default)]
 pub struct Map {
     selection: HashSet<Id>,
     map: String
 }
    impl Debug for Map {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            write!(f, "Map: <m>ap: {:?} }}", self.map);
        }
    }