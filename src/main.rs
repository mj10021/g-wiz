use std::f32::EPSILON;

use bevy::prelude::*;
use print_analyzer::{Parsed, Pos, Uuid};

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
    e: f32,
    selected: bool,
}
impl Extrusion {
    fn from_vertex(gcode: &Parsed, vertex: &Uuid) -> Extrusion {
        let v = gcode.vertices.get(vertex).unwrap();
        let Pos {
            x: xi,
            y: yi,
            z: zi,
            ..
        } = if let Some(prev) = v.prev {
            gcode.vertices.get(&prev).unwrap().to
        } else {
            Pos::unhomed()
        };
        let Pos {
            x: xf,
            y: yf,
            z: zf,
            e,
            ..
        } = v.to;
        Extrusion {
            id: v.id,
            xi,
            yi,
            zi,
            xf,
            yf,
            zf,
            e,
            selected: false,
        }
    }
}
fn draw_extrustions(gcode: Res<GCode>, mut commands: Commands) {
    let gcode = &gcode.0;
    for vertex in gcode.vertices.keys() {
        commands.spawn(Extrusion::from_vertex(&gcode, vertex));
    }
}

fn draw_cylinders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Extrusion>,
) {
    for extrusion in query.iter() {
        if extrusion.e < EPSILON { continue; }
        let start = Vec3::new(extrusion.xi, extrusion.yi, extrusion.zi);
        let end = Vec3::new(extrusion.xf, extrusion.yf, extrusion.zf);

        // Create a cylinder mesh
        let radius = 0.1;
        let length = start.distance(end);
        let cylinder = shape::Cylinder {
            radius,
            height: length,
            ..Default::default()
        };

        // Create the mesh and material
        let mesh_handle = meshes.add(cylinder);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..Default::default()
        });

        // Calculate the middle point and orientation of the cylinder
        let middle = (start + end) / 2.0;
        let direction = end - start;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

        // Add the cylinder to the scene
        commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            transform: Transform {
                translation: middle,
                rotation,
                ..Default::default()
            },
            ..Default::default()
        });
    }
}
fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 0.02,
    });
    // Add a light source
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
        ..default()
    });
    let zoom = 35.0;
    // Add a camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(5.0* zoom, 5.0*zoom, 5.0*zoom).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
fn main() {
    App::new()
        .insert_resource(GCode(
            print_analyzer::read("../print_analyzer/test.gcode", false).expect("failed to read"),
        ))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Startup, draw_extrustions)
        .add_systems(Update, draw_cylinders)
        .run();
}
