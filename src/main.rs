mod pan_orbit;
mod print_analyzer;
mod ui;

use bevy::math::primitives::Cylinder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_mod_picking::{debug::PointerDebug, prelude::*};
use pan_orbit::{pan_orbit_camera, PanOrbitCamera};
use picking_core::PickingPluginsSettings;
use print_analyzer::{Emit, Parsed, Pos};
use selection::send_selection_events;
use std::collections::HashMap;
use ui::*;
use uuid::Uuid;

#[derive(Default, Resource)]
struct IdMap(HashMap<Uuid, Entity>);

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
    mut map: ResMut<IdMap>,
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
        let e_id = commands
            .spawn((
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
                Tag { id: id.clone() },
            ))
            .id();
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
            Tag { id: id.clone() },
        ));
        map.0.insert(id.clone(), e_id);
    }
    commands.remove_resource::<ForceRefresh>();
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
    commands.init_resource::<UiResource>();
    commands.init_resource::<IdMap>();
    commands.init_resource::<EnablePanOrbit>();
}

fn display_information(hover: Query<(&PickingInteraction, &Tag)>, gcode: Res<GCode>) {
    for (interaction, tag) in hover.iter() {
        if *interaction == PickingInteraction::Hovered {
            let v = gcode.0.vertices.get(&tag.id).unwrap();
        }
    }
}

/// Update entity selection component state from pointer events.
fn update_selections(
    mut selectables: Query<(&mut PickSelection, &Tag)>,
    select_type: Res<UiResource>,
    mut selections: EventReader<Pointer<Select>>,
    mut deselections: EventReader<Pointer<Deselect>>,
    gcode: Res<GCode>,
    map: Res<IdMap>,
) {
    let select_type = select_type.selection_enum;
    if select_type == Choice::Vertex {
        return;
    }
    for selection in selections.read() {
        if let Ok((_, id)) = selectables.get_mut(selection.target) {
            if select_type == Choice::Shape {
                for id in gcode.0.get_shape(&id.id) {
                    let Some(entity) = map.0.get(&id) else {
                        continue;
                    };
                    {
                        let (mut select_me, _) =
                            selectables.get_mut(*entity).expect("entity not found");
                        select_me.is_selected = true;
                    }
                }
            } else if select_type == Choice::Layer {
                for id in gcode.0.get_layer(&id.id) {
                    let entity = map.0.get(&id).unwrap();
                    let (mut select_me, _) =
                        selectables.get_mut(*entity).expect("entity not found");
                    select_me.is_selected = true;
                }
            }
        }
    }
    for deselection in deselections.read() {
        if let Ok((_, id)) = selectables.get_mut(deselection.target) {
            if select_type == Choice::Shape {
                for id in gcode.0.get_shape(&id.id) {
                    let Some(entity) = map.0.get(&id) else {
                        continue;
                    };
                    let (mut deselect_me, _) =
                        selectables.get_mut(*entity).expect("entity not found");
                    deselect_me.is_selected = false;
                }
            } else if select_type == Choice::Layer {
                for id in gcode.0.get_layer(&id.id) {
                    let entity = map.0.get(&id).unwrap();
                    let (mut deselect_me, _) =
                        selectables.get_mut(*entity).expect("entity not found");
                    deselect_me.is_selected = false;
                }
            }
        }
    }
}

fn capture_mouse(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut pick_settings: ResMut<PickingPluginsSettings>,
    mut egui_context: Query<&mut EguiContext>,
) {
    let width = egui_context.single_mut().get_mut().used_rect().width();
    let Ok(window) = window.get_single() else {
        return;
    };
    if let Some(Vec2 { x, .. }) = window.cursor_position() {
        if x < width {
            pick_settings.is_enabled = false;
            commands.remove_resource::<EnablePanOrbit>();
        }
    }
}
fn reset_ui_hover(mut commands: Commands, mut pick_settings: ResMut<PickingPluginsSettings>) {
    commands.init_resource::<EnablePanOrbit>();
    pick_settings.is_enabled = true;
}

#[derive(Default, Resource)]
struct EnablePanOrbit;

fn update_visibilities(
    mut entity_query: Query<(&Tag, &mut Visibility)>,
    ui_res: Res<UiResource>,
    gcode: Res<GCode>,
) {
    let count = ui_res.vertex_counter;
    for (tag, mut vis) in entity_query.iter_mut() {
        if let Some(v) = gcode.0.vertices.get(&tag.id) {
            let selected = match v.label {
                print_analyzer::Label::PrePrintMove => ui_res.vis_select.preprint,
                print_analyzer::Label::PlanarExtrustion
                | print_analyzer::Label::NonPlanarExtrusion => ui_res.vis_select.extrusion,
                print_analyzer::Label::Retraction => ui_res.vis_select.retraction,
                print_analyzer::Label::DeRetraction => ui_res.vis_select.deretraction,
                print_analyzer::Label::Wipe => ui_res.vis_select.wipe,
                print_analyzer::Label::LiftZ | print_analyzer::Label::TravelMove => {
                    ui_res.vis_select.travel
                }
                _ => false,
            };
            if count > v.count && selected {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

fn main() {
    let gcode = print_analyzer::read(
        "../print_analyzer/Goblin Janitor_0.4n_0.2mm_PLA_MINIIS_10m.gcode",
        //"../print_analyzer/test.gcode",
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
            PreUpdate,
            (capture_mouse.before(send_selection_events)).chain(),
        )
        .add_systems(
            Update,
            (
                key_system,
                ui_example_system,
                capture_mouse,
                update_selections,
                update_visibilities,
            )
                .chain(),
        )
        .add_systems(
            Update,
            pan_orbit_camera.run_if(resource_exists::<EnablePanOrbit>),
        )
        .add_systems(Update, draw.run_if(resource_exists::<ForceRefresh>))
        .add_systems(PostUpdate, reset_ui_hover)
        .run();
}
