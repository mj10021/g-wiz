use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
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
    let pan_button = MouseButton::Middle;
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
            pan += ev.delta * zoom;
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
struct Extrusion {
    count: u32,
    xi: f32,
    yi: f32,
    zi: f32,
    xf: f32,
    yf: f32,
    zf: f32,
    e: f32,
    _selected: bool,
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
            count: v.count,
            xi,
            yi,
            zi,
            xf,
            yf,
            zf,
            e,
            _selected: false,
        }
    }
}
fn draw_extrustions(gcode: Res<GCode>, mut commands: Commands) {
    let gcode = &gcode.0;
    for vertex in gcode.vertices.keys() {
        commands.spawn(Extrusion::from_vertex(&gcode, vertex));
    }
}
#[derive(Component)]
struct Tag;

fn draw_cylinders(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    count: Res<VertexCounter>,
    query: Query<&Extrusion>,
    cylinders: Query<Entity, With<Tag>>
) {
    for cylinder in cylinders.iter() {
        commands.entity(cylinder).despawn();
    }
    
    for extrusion in query.iter() {
        if extrusion.e < EPSILON || extrusion.count > count.count {
            continue;
        }
        let start = Vec3::new(extrusion.xi, extrusion.yi, extrusion.zi);
        let end = Vec3::new(extrusion.xf, extrusion.yf, extrusion.zf);

        // Create a cylinder mesh
        let radius = 0.1;
        let length = start.distance(end);
        let cylinder = Cylinder {
            radius,
            half_height: length / 2.0,
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
        commands.spawn((PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            transform: Transform {
                translation: middle,
                rotation,
                ..Default::default()
            },
            ..Default::default()
        }, Tag));
        println!("drawn");
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
    let translation = Vec3::new(5.0 * zoom, 5.0 * zoom, 5.0 * zoom);
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
struct SecretCount(u32);

fn ui_example_system(mut contexts: EguiContexts, vertex: Res<VertexCounter>, mut counter: ResMut<SecretCount>) {
    let max = vertex.max;
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.add(egui::Slider::new(&mut counter.0, 0..=max));
        if ui.button("asdf").clicked() {
            println!("asdf")
        }
    });
}
fn update_count(secret: Res<SecretCount>, mut counter: ResMut<VertexCounter>) {
    if secret.0 != counter.count {
        counter.count = secret.0;
    }
}
fn main() {
    let gcode =
        print_analyzer::read("../print_analyzer/test.gcode", false).expect("failed to read");
    App::new()
        .insert_resource(VertexCounter::build(&gcode))
        .insert_resource(SecretCount(0))
        .insert_resource(GCode(gcode))
        .add_plugins((DefaultPlugins, EguiPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (ui_example_system, pan_orbit_camera, update_count).chain())
        .add_systems(Startup, draw_extrustions)
        .add_systems(
            Update,
            draw_cylinders.run_if(resource_changed::<VertexCounter>),
        )
        .run();
}
