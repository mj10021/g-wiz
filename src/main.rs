/*use bevy::math::primitives::Direction3d;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    //let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(ground.translation(), Plane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(
        point + ground.up() * 0.01,
        Direction3d::new_unchecked(ground.up()), // Up vector is already normalized.
        0.2,
        Color::WHITE,
    );
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(20., 20.)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            ..default()
        },
        Ground,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

*/use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy::render::view::window;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContexts, EguiPlugin};
use print_analyzer::{Parsed, Pos, Uuid};
use std::f32::EPSILON;

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
fn pan_orbit_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Left;
    let zoom = 35.0;
    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta * zoom * zoom;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button) || input_mouse.just_pressed(orbit_button) {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let Ok(window) = primary_query.get_single() else {
                panic!()
            };
            let delta_x = {
                let delta = rotation_move.x / window.width() * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.height() * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let Ok(window) = primary_query.get_single() else {
                panic!()
            }; //get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov)
                    / (window.height() * window.width());
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    ev_motion.clear();
}

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
        let radius = 0.1;
        let length = start.distance(end);
        let cylinder = Cylinder {
            radius,
            half_height: length / 2.0,
        };
        let sphere = Sphere {
            radius,
        };

        // Create the mesh and material
        let mesh_handle = meshes.add(cylinder);
        let sphere_handle = meshes.add(sphere);
        let material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.2),
            ..Default::default()
        });
        let sphere_material = materials.add( StandardMaterial {
            base_color: Color::rgb(0.0,0.8,0.0),
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

        ));
        commands.spawn((
            PbrBundle {
                mesh: sphere_handle,
                material: sphere_material,
                transform: Transform {
                    translation: end,
                    ..Default::default()
                },
                ..Default::default()
            },
            Tag(id.clone()),
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

fn ui_example_system(
    mut contexts: EguiContexts,
    vertex: Res<VertexCounter>,
    layer: Res<LayerCounter>,
    mut counter: ResMut<SecretCount>,
    mut layer_counter: ResMut<SecretLayerCount>,
) {
    let max = vertex.max;
    let layer_max = layer.max;
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
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
    windows: Query<&Window>,
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
        if dist < 500.0 {
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
        .insert_resource(SecretCount(0))
        .insert_resource(SecretLayerCount(0))
        .insert_resource(Selection(Uuid::nil()))
        .insert_resource(GCode(gcode))
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                key_system,
                ui_example_system,
                pan_orbit_camera,
                update_count,
                draw_cursor
            )
                .chain(),
        )
        .add_systems(Update, draw_cursor)
        .add_systems(
            Update,
            draw.run_if(resource_changed::<VertexCounter>),
        )
        .run();
}
