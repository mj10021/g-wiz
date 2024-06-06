use bevy::prelude::*;

#[derive(Component)]
struct Extrusion {
    xi: f32,
    yi: f32,
    zi: f32,
    xf: f32,
    yf: f32,
    zf: f32,
    e: f32
}
impl Extrusion {
    fn new() -> Extrusion {
        Extrusion {
            xi: 0.0,
            yi: 0.0,
            zi: 0.0,
            xf: 0.0,
            yf: 0.0,
            zf: 0.0,
            e: 0.0
        }
    }
}
fn draw_extrustions(mut commands: Commands) {
    commands.spawn(Extrusion::new());
}
fn main() {
    App::new()
        .add_systems(Update, draw_extrustions)
        .run();
}
