use super::{print_analyzer::Label, settings::*, GCode, IdMap, PickableBundle, Tag, UiResource};
use crate::callbacks::handlers::ForceRefresh;
use crate::BoundingBox;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, SpecializedMeshPipelineError,
        },
    },
};

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>,
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
pub struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    //fn fragment_shader() -> ShaderRef {
    //    "shaders/line_material.wgsl".into()
    //}

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

        Mesh::new(
            // This tells wgpu that the positions are list of lines
            // where every pair is a start and end point
            PrimitiveTopology::LineList,
            RenderAssetUsages::RENDER_WORLD,
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

pub fn setup_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut line_material: ResMut<Assets<LineMaterial>>,
    bounding_box: Res<BoundingBox>,
) {
    let mut lines = Vec::new();
    let border = 20;
    let step = 5;
    let (x_max, y_max, z_max) = (
        bounding_box.max.x as u32 + border,
        bounding_box.max.y as u32 + border,
        bounding_box.max.z as u32 + border,
    );
    let (x_min, y_min, z_min) = (
        bounding_box.min.x as u32 - border,
        bounding_box.min.y as u32 - border,
        bounding_box.min.z as u32,
    );
    // FIXME: make this casting beter
    for x in (x_min..=x_max).step_by(step) {
        let start = Vec3::new(x as f32, y_min as f32, z_min as f32);
        let end = Vec3::new(x as f32, y_max as f32, z_min as f32);
        lines.push((start, end));
        let start = Vec3::new(x as f32, y_max as f32, z_min as f32);
        let end = Vec3::new(x as f32, y_max as f32, z_max as f32);
        lines.push((start, end));
    }
    for y in (y_min..=y_max).step_by(step) {
        let start = Vec3::new(x_min as f32, y as f32, z_min as f32);
        let end = Vec3::new(x_max as f32, y as f32, z_min as f32);
        lines.push((start, end));
        let start = Vec3::new(x_max as f32, y as f32, z_min as f32);
        let end = Vec3::new(x_max as f32, y as f32, z_max as f32);
        lines.push((start, end));
        for z in (z_min..=z_max).step_by(step) {
            let start = Vec3::new(x_min as f32, y_max as f32, z as f32);
            let end = Vec3::new(x_max as f32, y_max as f32, z as f32);
            lines.push((start, end));
            let start = Vec3::new(x_max as f32, y_min as f32, z as f32);
            let end = Vec3::new(x_max as f32, y_max as f32, z as f32);
            lines.push((start, end));
        }
    }
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(LineList { lines }),
        material: line_material.add(LineMaterial {
            color: Color::GREEN,
        }),
        ..default()
    });
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
