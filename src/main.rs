mod pan_orbit;
mod ui;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::prelude::*;
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use print_analyzer::{Parsed, Pos, Uuid};
use std::f32::EPSILON;
use ui::*;

#[derive(Resource)]
struct GCode(Parsed);

#[derive(Component)]
struct Tag{
    id: Uuid
}



fn draw(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    count: Res<VertexCounter>,
    gcode: Res<GCode>,
    cylinders: Query<Entity>,
) {
    for cylinder in cylinders.iter() {
        commands.entity(cylinder).despawn();
    }
    let gcode = &gcode.0;

    for (id, vertex) in gcode.vertices.iter() {
        let Pos {
            x: xf,
            y: yf,
            z: zf,
            e,
            ..
        } = vertex.to;
        let (xi, yi, zi) = {
            if let Some(prev) = vertex.prev {
                let p = gcode.vertices.get(&prev).unwrap();
                (p.to.x, p.to.y, p.to.z)
            } else {
                (0.0, 0.0, 0.0)
            }
        };

        if e < EPSILON || vertex.count > count.count {
            continue;
        }
        let start = Vec3::new(xi, yi, zi);
        let end = Vec3::new(xf, yf, zf);

        // Create a cylinder mesh
        let radius = 0.05;
        let length = start.distance(end);
        let cylinder = Cylinder {
            radius,
            half_height: length / 2.0,
        };
        let sphere = Sphere {
            radius: radius * 1.618,
        };

        // Create the mesh and material
        let mesh_handle = meshes.add(cylinder);
        let sphere = meshes.add(sphere);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::ORANGE_RED,
            ..Default::default()
        });
        let material_handle2 = materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..Default::default()
        });

        // Calculate the middle point and orientation of the cylinder
        let middle = (start + end) / 2.0;
        let direction = end - start;
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
        // Add the cylinder to the scene
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform {
                    translation: middle,
                    rotation,
                    ..Default::default()
                },
                ..Default::default()
            },
            PickableBundle::default(),
            Tag {id: id.clone()},
        ));
        commands.spawn((
            PbrBundle {
                mesh: sphere,
                material: material_handle2,
                transform: Transform {
                    translation: middle,
                    rotation,
                    ..Default::default()
                },
                ..Default::default()
            },
            PickableBundle::default(),
            Tag{id: id.clone()},
        ));
    }
}
fn selection_query (mut commands: Commands, mut s_query: Query<(&PickSelection, &mut Tag)>, mut selection: ResMut<Selection>) {
    for (s, tag) in s_query.iter_mut() {
        if !s.is_selected { continue; } else {
            selection.0 = tag.id;
        }
    }

}
fn setup(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 100.0,
    });
    let zoom = 35.0;
    let translation = Vec3::new(5.0 * zoom, -5.0 * zoom, 5.0 * zoom);
    let radius = translation.length();

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
    ));
}

#[derive(Resource)]
struct Selection(Uuid);
impl Default for Selection {
    fn default() -> Self {
        Selection(Uuid::nil())
    }
}
fn main() {
    // let args: Vec<String> = env::args().collect();
    let path = "../print_analyzer/Goblin Janitor_0.4n_0.2mm_PLA_MINIIS_10m.gcode";
    let gcode = print_analyzer::read(
        path,
        false,
    )
    .expect("failed to read");
    App::new()
        .add_plugins((DefaultPlugins, DefaultPickingPlugins, EguiPlugin))
        .init_resource::<SecretCount>()
        .init_resource::<SecretLayerCount>()
        .init_resource::<Selection>()
        .init_resource::<Enum>()
        .insert_resource(VertexCounter::build(&gcode))
        .insert_resource(LayerCounter::build(&gcode))
        .insert_resource(GCode(gcode))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                key_system,
                ui_example_system,
                pan_orbit_camera,
                update_count,
            )
                .chain(),
        )
        .add_systems(Update, draw.run_if(resource_changed::<VertexCounter>))
        .run();
}
