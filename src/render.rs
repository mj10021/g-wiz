use super::{
    print_analyzer::Label, settings::*, ForceRefresh, GCode, IdMap, PickableBundle, Tag, UiResource,
};
use bevy::prelude::*;
use bevy_mod_picking::{
    focus::PickingInteraction, highlight::PickHighlight, selection::PickSelection,
};
use std::collections::HashSet;



pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<IdMap>,
    gcode: Res<GCode>,
    shapes: Query<Entity, With<Tag>>,
    settings: Res<Settings>,
) {
    for shape in shapes.iter() {
        commands.entity(shape).despawn();
    }
    let gcode = &gcode.0;
    let mut pos_list = Vec::new();
    for v in gcode.vertices.values() {
        let (xf, yf, zf) = (v.to.x, v.to.y, v.to.z);
        let (xi, yi, zi) = {
            if let Some(prev) = v.prev {
                let p = gcode.vertices.get(&prev).unwrap();
                (p.to.x, p.to.y, p.to.z)
            } else {
                (0.0, 0.0, 0.0)
            }
        };
        let (start, end) = (Vec3::new(xi, yi, zi), Vec3::new(xf, yf, zf));
        let dist = start.distance(end);
        let flow = v.to.e / dist;
        pos_list.push((v.id, start, end, flow));
    }
    for (id, start, end, flow) in pos_list {
        // Create a cylinder mesh
        let radius = (flow / std::f32::consts::PI).sqrt();

        // Create the mesh and material
        let sphere_material = materials.add(StandardMaterial {
            base_color: settings.extrusion_node_color,
            ..Default::default()
        });

        // Create a cylinder mesh
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
            base_color: settings.extrusion_color,
            emissive: settings.extrusion_color,
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
                Tag { id },

            ))
            .id();
        // add the
        commands.spawn((
            PbrBundle {
                mesh: sphere,
                material: sphere_material,
                transform: Transform {
                    translation: end,
                    ..Default::default()
                },
                ..Default::default()
            },
            PickableBundle::default(),
            Tag { id },
        ));
        map.0.insert(id, e_id);
    }
    commands.remove_resource::<ForceRefresh>();
}

pub fn update_visibilities(
    mut entity_query: Query<(&Tag, &mut Visibility)>,
    ui_res: Res<UiResource>,
    gcode: Res<GCode>,
) {
    let count = ui_res.vertex_counter;
    for (tag, mut vis) in entity_query.iter_mut() {
        if let Some(v) = gcode.0.vertices.get(&tag.id) {
            let selected = match v.label {
                Label::PrePrintMove => ui_res.vis_select.preprint,
                Label::PlanarExtrustion | Label::NonPlanarExtrusion => ui_res.vis_select.extrusion,
                Label::Retraction => ui_res.vis_select.retraction,
                Label::DeRetraction => ui_res.vis_select.deretraction,
                Label::Wipe => ui_res.vis_select.wipe,
                Label::LiftZ | Label::TravelMove => ui_res.vis_select.travel,
                _ => false,
            };
            if count > v.count
                && selected
                && v.to.z < ui_res.display_z_max.0
                && v.to.z > ui_res.display_z_min
            {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

pub fn match_objects(mut p_query: Query<(&mut PickingInteraction, Entity, &Tag)>, id_map: Res<IdMap>) {
    let mut p_map: std::collections::HashMap<Tag, PickingInteraction> = std::collections::HashMap::new();
    let ids = id_map.0.values().collect::<HashSet<_>>();
    for (p, e, t) in p_query.iter_mut() {
        if ids.contains(&e) {
            p_map.insert(*t, *p);
        }
    }
    for (mut p, _, t) in p_query.iter_mut() {
        *p = *p_map.get(t).unwrap();
    }
}
