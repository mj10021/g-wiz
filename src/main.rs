mod pan_orbit;
mod ui;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseWheel};
use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiPlugin;
use bevy_mod_picking::prelude::*;
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use print_analyzer::{Parsed, Pos, Uuid};
use std::collections::HashSet;
use ui::*;

#[derive(Resource)]
struct GCode(Parsed);

#[derive(Default, Resource)]
struct ForceRefresh;

#[derive(Component)]
struct Tag {
    id: Uuid,
}

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
        let Pos {
            x: xf,
            y: yf,
            z: zf,
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

        if !vertex.is_extrusion() || vertex.count > count.count {
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
            NoDeselect,
            Tag { id: id.clone() },
        ));
        commands.spawn((
            PbrBundle {
                mesh: sphere,
                material: material_handle2,
                transform: Transform {
                    translation: end,
                    ..Default::default()
                },
                ..Default::default()
            },
            PickableBundle::default(),
            NoDeselect,
            Tag { id: id.clone() },
        ));
    }
    commands.remove_resource::<ForceRefresh>();
}
fn selection_query(
    mut s_query: Query<(&mut PickSelection, &mut Tag)>,
    ui_res: Res<UiResource>,
    gcode: Res<GCode>,
    mut selection: ResMut<Selection>,
) {
    for (mut s, tag) in s_query.iter_mut() {
        if !s.is_selected {
            if selection.0.contains(&tag.id) {
                s.is_selected = true;
            }
            continue;
        } else {
            if !selection.0.contains(&tag.id) {
                match ui_res.selection_enum {
                    Choice::Vertex => {
                        selection.0.insert(tag.id);
                    }
                    Choice::Shape => {
                        selection.0.extend(gcode.0.get_shape(&tag.id));
                    }
                    Choice::Layer => {
                        selection.0.extend(gcode.0.get_shape(&tag.id));
                    }
                };
            }
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
    commands.init_resource::<ForceRefresh>();
    commands.init_resource::<Selection>();
    commands.init_resource::<UiResource>()
}
fn mouse_input_system(
    egui_context_q: Res<UiResource>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut mouse_button_input_events: ResMut<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    let (width, height) = egui_context_q.panel_size;
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let Some(Vec2 { x, y }) = window.cursor_position() else {
        return;
    };
    if x < width && y < height {
        // Clear mouse input events if egui is handling them
        mouse_button_input_events.clear();
        mouse_motion_events.clear();
        mouse_wheel_events.clear();
    }
}
#[derive(Resource)]
struct Selection(HashSet<Uuid>);
impl Default for Selection {
    fn default() -> Self {
        Selection(HashSet::new())
    }
}
impl Selection {
    fn reset_selection(&mut self, mut s_query: Query<&mut PickSelection>) {
        for mut s in s_query.iter_mut() {
            s.is_selected = false;
        }
        self.0 = HashSet::new();
    }
}
fn main() {
    let gcode = print_analyzer::read(
        "../print_analyzer/Goblin Janitor_0.4n_0.2mm_PLA_MINIIS_10m.gcode",
        false,
    )
    .expect("failed to read");
    App::new()
        .add_plugins((DefaultPlugins, DefaultPickingPlugins, EguiPlugin))
        .insert_resource(VertexCounter::build(&gcode))
        .insert_resource(LayerCounter::build(&gcode))
        .insert_resource(GCode(gcode))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                key_system,
                ui_example_system,
                mouse_input_system,
                pan_orbit_camera,
                update_counts,
                selection_query,
            )
                .chain(),
        )
        .add_systems(Update, draw.run_if(resource_exists::<ForceRefresh>))
        .run();
}
