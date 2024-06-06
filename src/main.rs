use bevy::{prelude::*, transform::commands};
use print_analyzer::{Parsed, Uuid, Pos};

#[derive(Resource)]
struct GCode(Parsed);

#[derive(Component)]
struct Extrusion {
    id: Uuid,
    xi: f32,
    yi: f32,
    zi: f32,
    xf: f32,
    yf: f32,
    zf: f32,
    e: f32
}
impl Extrusion {
    fn from_vertex(gcode: &Parsed, vertex: &Uuid) -> Extrusion {
        let v = gcode.vertices.get(vertex).unwrap();
        let prev = gcode.vertices.get(&v.prev.unwrap()).unwrap();
        let Pos {x: xi, y: yi, z: zi, ..} = prev.to;
        let Pos {x: xf, y: yf, z: zf, e, ..} = v.to;
        Extrusion { id: v.id, xi, yi, zi, xf, yf, zf, e }
    }
}
fn load_gcode(gcode: Res<GCode>, mut commands: Commands) {

}
fn draw_extrustions(gcode: Res<GCode>, mut commands: Commands) {
    let gcode = &gcode.0;
    for vertex in gcode.vertices.keys() {
        commands.spawn(Extrusion::from_vertex(&gcode, vertex));
    }
}
fn main() {
    let gcode = Parsed::new();
    App::new()
        .insert_resource(GCode(gcode))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, load_gcode)
        .add_systems(Update, draw_extrustions)
        .run();
}
