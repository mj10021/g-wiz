use super::{
    print_analyzer::Label, settings::*, ForceRefresh, GCode, IdMap, PickableBundle, Tag, UiResource,
};
use bevy::prelude::*;

pub fn setup_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    gcode: Res<GCode>,
) {
    let (mut x_min, mut y_min, mut z_min): (f32, f32, f32) = (0.0, 0.0, 0.0);
    let (mut x_max, mut y_max, mut z_max): (f32, f32, f32) = (500.0, 500.0, 500.0);
    for v in gcode.0.vertices.values() {
        if !v.extrusion_move() {
            continue;
        }
        x_min = x_min.min(v.to.x);
        y_min = y_min.min(v.to.y);
        z_min = z_min.min(v.to.z);
        x_max = x_max.max(v.to.x);
        y_max = y_max.max(v.to.y);
        z_max = z_max.max(v.to.z);
    }
    let origin = Vec3 {
        x: x_min,
        y: y_min,
        z: z_min,
    };
    let pt1 = Vec3::new(x_max, y_min, z_min);
    let pt2 = Vec3::new(x_min, y_max, z_min);
    let max_pt = Vec3::new(x_max, y_max, z_min);
    let w = x_max - x_min;
    let l = y_max - y_min;
    let h = z_max - z_min;
    // FIXME: make these lines
    let _ = commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_corners(origin, max_pt)),
        material: materials.add(StandardMaterial {
            base_color: Color::GRAY,
            ..Default::default()
        }),
        transform: Transform {
            translation: Vec3::new(w / 2.0, l/2.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
    // let _ = commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::from_corners(origin, pt2)),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::GRAY,
    //         ..Default::default()
    //     }),
    //     transform: Transform {
    //         translation: Vec3::new(0.0, l / 2.0, 0.0),
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });
    // let _ = commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::from_corners(origin, max_pt)),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::GRAY,
    //         ..Default::default()
    //     }),
    //     transform: Transform {
    //         translation: Vec3::new(w / 2.0, l / 2.0, h / 2.0),
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // });
}

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
        pos_list.push((v.id, start, end, flow, v.label));
    }
    for (id, start, end, flow, label) in pos_list {
        if label == Label::FeedrateChangeOnly || label == Label::Home || label == Label::MysteryMove
        {
            continue;
        }
        let radius = (flow / std::f32::consts::PI).sqrt();
        let length = start.distance(end);
        let direction = end - start;
        let mut sphere = false;

        // Create the mesh and material
        let mesh_handle = match label {
            Label::PlanarExtrustion | Label::NonPlanarExtrusion | Label::PrePrintMove => meshes
                .add(Cylinder {
                    radius,
                    half_height: length / 2.0,
                }),
            Label::TravelMove | Label::LiftZ | Label::LowerZ | Label::Wipe => {
                meshes.add(Cylinder {
                    radius: 0.1,
                    half_height: length / 2.0,
                })
            }
            Label::DeRetraction | Label::Retraction => meshes.add(Sphere { radius: 0.6 }),
            _ => {
                panic!("{:?}", label)
            }
        };
        if label == Label::DeRetraction || label == Label::Retraction {
            sphere = true;
        }
        let material_handle = match label {
            Label::PlanarExtrustion | Label::NonPlanarExtrusion | Label::PrePrintMove => materials
                .add(StandardMaterial {
                    base_color: settings.extrusion_color,
                    ..Default::default()
                }),
            Label::TravelMove | Label::LiftZ | Label::LowerZ | Label::Wipe => {
                materials.add(StandardMaterial {
                    base_color: settings.travel_color,
                    ..Default::default()
                })
            }
            Label::DeRetraction => materials.add(StandardMaterial {
                base_color: settings.deretraction_color,
                ..Default::default()
            }),
            Label::Retraction => materials.add(StandardMaterial {
                base_color: settings.retraction_color,
                ..Default::default()
            }),
            _ => panic!(),
        };

        // Calculate the middle point and orientation of the cylinder
        let rotation = if sphere {
            Transform::default().rotation
        } else {
            Quat::from_rotation_arc(Vec3::Y, direction.normalize())
        };
        let translation = if sphere { end } else { (start + end) / 2.0 };
        let e_id = commands
            .spawn((
                PbrBundle {
                    mesh: mesh_handle,
                    material: material_handle.clone(),
                    transform: Transform {
                        translation,
                        rotation,
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
