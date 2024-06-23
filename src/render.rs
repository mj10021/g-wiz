use super::{
    print_analyzer::Label, ForceRefresh, GCode, IdMap, PickableBundle, Pos, Tag, UiResource,
};
use bevy::prelude::*;

pub fn render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: ResMut<IdMap>,
    gcode: Res<GCode>,
    shapes: Query<Entity, With<Tag>>,
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
            } else {(0.0, 0.0, 0.0)}
        };
        pos_list.push((v.id, Vec3::new(xi, yi, zi), Vec3::new(xf, yf, zf)));
    }
    for (id, start, end) in pos_list {

        // Create a cylinder mesh
        let sphere = Sphere {
            radius: 0.125,
        };

        // Create the mesh and material
        //let mesh_handle = meshes.add(cylinder);
        let sphere = meshes.add(sphere);
        let line_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.0, 1.0, 0.0),
            emissive: Color::rgb(0.0, 1.0, 0.0),
            ..Default::default()
        });
        let sphere_material = materials.add(StandardMaterial {
            base_color: Color::BLUE,
            ..Default::default()
        });

        // Add the move line to the scene
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(
                    Segment3d::from_points(start, end).0,
                ),
                material: line_material,
                ..Default::default()
            },
            Tag { id },
        ));
        // add the
        let e_id = commands
            .spawn((
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
            ))
            .id();
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
