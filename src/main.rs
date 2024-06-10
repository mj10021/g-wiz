mod pan_orbit;
use pan_orbit::{PanOrbitCamera, pan_orbit_camera};
use bevy_mod_picking::prelude::*;
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin};
use print_analyzer::{Parsed, Pos, Uuid};
use std::f32::EPSILON;

#[derive(Resource)]
struct GCode(Parsed);

#[derive(Component)]
struct Tag(Uuid);

fn draw(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    count: Res<VertexCounter>,
    gcode: Res<GCode>,
    cylinders: Query<Entity, With<Tag>>,
) {
    for cylinder in cylinders.iter() {
        commands.entity(cylinder).despawn();
    }
    let gcode = &gcode.0;

    for (id, vertex) in gcode.vertices.iter() {

        let Pos {x: xf, y: yf, z: zf, e, ..} = vertex.to;
        let (xi, yi, zi) = {
            if let Some(prev) = vertex.prev {
                let p = gcode.vertices.get(&prev).unwrap();
                (p.to.x, p.to.y, p.to.z)
            } else {(0.0, 0.0, 0.0)}
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
            Tag(id.clone()),
            On::<Pointer<Click>>::target_component_mut::<Visibility>(|click, visibility| {*visibility = Visibility::Hidden;})
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
            Tag(id.clone()),
            On::<Pointer<Click>>::target_component_mut::<Visibility>(|click, visibility| {*visibility = Visibility::Hidden;})
        ));
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
struct VertexCounter {
    count: u32,
    max: u32,
}

impl VertexCounter {
    fn build(gcode: &Parsed) -> VertexCounter {
        let max = gcode.vertices.keys().len() as u32;
        VertexCounter { count: 0, max }
    }
}
#[derive(Resource)]
struct LayerCounter {
    count: u32,
    max: u32,
}

impl LayerCounter {
    fn build(gcode: &Parsed) -> LayerCounter {
        let max = gcode.layers.len() as u32;
        LayerCounter { count: 0, max }
    }
}
#[derive(Resource)]
struct SecretCount(u32);

#[derive(Resource)]
struct SecretLayerCount(u32);

#[derive(Resource)]
struct Selection(Uuid);

fn key_system(
    mut counter: ResMut<SecretCount>,
    mut layer_counter: ResMut<SecretLayerCount>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.pressed(KeyCode::ArrowLeft) {
        counter.0 -= 1;
    } else if keys.pressed(KeyCode::ArrowRight) {
        counter.0 += 1;
    } else if keys.pressed(KeyCode::ArrowUp) {
        layer_counter.0 += 1;
    } else if keys.pressed(KeyCode::ArrowDown) {
        layer_counter.0 -= 1;
    }
}
#[derive(PartialEq)]
enum Choice { Vertex, Shape, Layer }

#[derive(Resource)]
struct Enum(Choice);


fn ui_example_system(
    mut contexts: EguiContexts,
    vertex: Res<VertexCounter>,
    layer: Res<LayerCounter>,
    mut counter: ResMut<SecretCount>,
    mut layer_counter: ResMut<SecretLayerCount>,
    mut enu: ResMut<Enum>
) {
    let max = vertex.max;
    let layer_max = layer.max;
    egui::SidePanel::new(egui::panel::Side::Left, "panel").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.add(egui::Slider::new(&mut counter.0, 0..=max));
        ui.add(egui::Slider::new(&mut layer_counter.0, 0..=layer_max).vertical());
        let steps = [
            (100, "<<<"),
            (10, "<<"),
            (1, "<"),
            (1, ">"),
            (10, ">>"),
            (100, ">>>"),
        ];
        let mut i = 0;
        egui::Grid::new("vertex stepper")
            .min_col_width(4.0)
            .show(ui, |ui| {
                for (num, str) in steps {
                    let neg = i < steps.len() / 2;
                    if ui.button(str).clicked() {
                        if neg {
                            counter.0 -= num;
                        } else {
                            counter.0 += num;
                        }
                    }
                    i += 1;
                }
                ui.end_row();
            });

        ui.horizontal(|ui| {
            ui.radio_value(&mut enu.0, Choice::Vertex, "Vertex");
            ui.radio_value(&mut enu.0, Choice::Shape, "Shape");
            ui.radio_value(&mut enu.0, Choice::Layer, "Layer");
        });

    });
}
fn update_count(secret: Res<SecretCount>, mut counter: ResMut<VertexCounter>) {
    if secret.0 as u32 != counter.count {
        counter.count = secret.0 as u32;
    }
}

fn draw_cursor(
    camera_query: Query<(&Camera, &Transform, &GlobalTransform)>,
    gcode: Res<GCode>,
    mut selection: ResMut<Selection>,
    mut counter: ResMut<VertexCounter>,
    spheres: Query<(&Transform, &Tag)>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let (camera, pos,camera_transform) = camera_query.single();
    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };
    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let mut hits = Vec::new();
    let pos = pos.translation;
    for sphere in spheres.iter() {
        let dist = sphere.0.translation.distance(pos);
        if ray.get_point(dist).distance(pos) < 10.0 {
            hits.push((sphere.1.0, dist));
        }
    }
    if hits.len() > 0 {
        hits.sort_by_key(|v| v.1 as i32);
        if selection.0 != hits[0].0 {
            selection.0 = hits[0].0;
            counter.count = counter.count.clone();
            println!("HIT");
        }
    }
}


fn main() {
    let gcode =
        print_analyzer::read("../print_analyzer/Goblin Janitor_0.4n_0.2mm_PLA_MINIIS_10m.gcode", false).expect("failed to read");
    App::new()
        .insert_resource(VertexCounter::build(&gcode))
        .insert_resource(LayerCounter::build(&gcode))
        .insert_resource(Enum(Choice::Vertex))
        .insert_resource(SecretCount(0))
        .insert_resource(SecretLayerCount(0))
        .insert_resource(Selection(Uuid::nil()))
        .insert_resource(GCode(gcode))
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                key_system,
                ui_example_system,
                pan_orbit_camera,
                update_count
            )
                .chain(),
        )
        .add_systems(
            Update,
            draw.run_if(resource_changed::<VertexCounter>),
        )
        .run();
}
