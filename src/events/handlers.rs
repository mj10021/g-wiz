use super::{events::*, console::*};
use crate::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use bevy_mod_picking::prelude::*;
use egui::Pos2;

#[derive(Default, Resource)]
pub struct ForceRefresh;

#[derive(Resource, PartialEq)]
pub struct PanOrbit(pub bool);

impl Default for PanOrbit {
    fn default() -> Self {
        Self(true)
    }
}

pub fn ui_handler(
    mut event: EventReader<UiEvent>,
    mut ui_res: ResMut<UiResource>,
    mut s_query: Query<&mut PickSelection>,
    mut pan_orbit: ResMut<PanOrbit>,
    mut command_writer: EventWriter<CommandEvent>,
) {
    for event in event.read() {
        match event {
            UiEvent::MoveDisplay(forward, layer, count) => {
                if *layer && *forward {
                    ui_res.display_z_max.1 += count;
                } else if *layer {
                    ui_res.display_z_max.0 += count;
                } else if *forward {
                    ui_res.vertex_counter += *count as u32;
                } else {
                    if ui_res.vertex_counter == 0 {
                        return;
                    }
                    ui_res.vertex_counter -= *count as u32;
                }
            }
            UiEvent::SelectAll => {
                let select_all = s_query.iter().any(|s| !s.is_selected);
                for mut selection in s_query.iter_mut() {
                    selection.is_selected = select_all;
                }
            }
            UiEvent::SetPanOrbit(on) => {
                pan_orbit.0 = *on;
            }
            UiEvent::ConsoleEnter(s) => {
                if let Ok(event) = CommandEvent::build(s) {
                    command_writer.send(event);
                }
            },
            UiEvent::ConsoleResponse(s) => todo!(),
            UiEvent::CommandEnter => todo!(),
            UiEvent::MergeDelete => todo!(),
            UiEvent::HoleDelete => todo!(),
            UiEvent::Undo => todo!(),
            UiEvent::Redo => todo!(),
        }
    }
}

#[derive(Default, Resource)]
pub struct ExportDialogue(pub bool);

fn system_handler(
    mut events: EventReader<SystemEvent>,
    mut commands: Commands,
    mut egui_context: Query<&mut EguiContext>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut path: ResMut<FilePath>,
    mut open: ResMut<ExportDialogue>,
    gcode: Res<GCode>,
) {
    for event in events.read() {
        match event {
            SystemEvent::Open => {
                // do something
            }
            SystemEvent::Save => {
                // do something
            }
            SystemEvent::SaveAs => {
                if let Ok(window) = window.get_single() {
                    let x = window.width() / 2.0;
                    let y = window.height() / 3.0;
                    if let Ok(mut context) = egui_context.get_single_mut() {
                        egui::containers::Window::new("Export as...")
                            .open(&mut open.0)
                            .default_pos(Pos2 { x, y })
                            .collapsible(false)
                            .show(context.get_mut(), |ui| {
                                ui.label("Path:");
                                ui.text_edit_singleline(&mut path.0);
                                if ui.button("Export").clicked() {
                                    let path = std::path::PathBuf::from(path.0.clone());
                                    if let Some(path) = path.to_str() {
                                        let _ = gcode.0.write_to_file(path);
                                    }
                                }
                            });
                    }
                }
            }
            SystemEvent::RecalcBounds => {
                // do something
            }
            SystemEvent::ForceRefresh => {
                commands.init_resource::<ForceRefresh>();
            }
        }
    }
}

pub fn command_handler(
    mut gcode: ResMut<GCode>,
    s_query: Query<(&PickSelection, &Tag)>,
    mut bounding_box: ResMut<crate::BoundingBox>,
    mut refresh: EventWriter<SystemEvent>,
    mut event: EventReader<CommandEvent>,
    ui_res: Res<UiResource>,
) {
    let count = ui_res.subdivide_slider;
    let mut selection = s_query
        .iter()
        .filter_map(|(s, t)| if !s.is_selected { None } else { Some(t.id) })
        .collect();
    let centroid = gcode.0.get_centroid(&selection);
    for event in event.read() {
        // need to wait for updates/confirm from the console to actually apply the transformation
        match event {
            CommandEvent::Subdivide(subdivide) => {
                gcode.0.subdivide_vertices(selection.clone(), count);
            }
            CommandEvent::Draw(draw) => {
                todo!();
            }
            CommandEvent::Translate(translate) => {
                todo!()
                // let vec = Vec3::from(translate)
                // for id in selection.iter() {
                //     gcode.0.translate(id, translate);
                // }
            }
            CommandEvent::Rotate(rotate) => {
                for id in selection.iter() {
                    gcode.0.rotate(
                        id,
                        &centroid,
                        &Vec3::new(
                            rotate.rho.unwrap_or(0.0),
                            rotate.theta.unwrap_or(0.0),
                            rotate.phi.unwrap_or(0.0),
                        ),
                    );
                }
            }
            CommandEvent::Scale(scale) => {
                for id in selection.iter() {
                    gcode.0.scale(
                        id,
                        &centroid,
                        &Vec3::new(
                            scale.x.unwrap_or(1.0),
                            scale.y.unwrap_or(1.0),
                            scale.z.unwrap_or(1.0),
                        ),
                    );
                }
            }
            CommandEvent::Filter(filter) => {
                todo!();
            }
            CommandEvent::Map(map) => {
                todo!();
            }
        }
        refresh.send(SystemEvent::ForceRefresh);
    }
}

pub fn selection_handler(
    mut select_reader: EventReader<Pointer<Select>>,
    mut deselect_reader: EventReader<Pointer<Deselect>>,
    ui_res: Res<UiResource>,
    mut selectables: Query<(&mut PickSelection, &Tag)>,
    map: Res<IdMap>,
    gcode: Res<GCode>,
) {
    let select_ids = select_reader
        .read()
        .map(|s| selectables.get(s.target).unwrap().1.id);
    let deselect_ids = deselect_reader
        .read()
        .map(|s| &selectables.get(s.target).unwrap().1.id);
    let events = {
        let mut out = (Vec::new(), Vec::new());
        match ui_res.selection_enum {
            Choice::Vertex => {}
            Choice::Shape => {
                for id in select_ids {
                    out.0.append(&mut gcode.0.get_shape(&id));
                }
                for id in deselect_ids {
                    out.1.append(&mut gcode.0.get_shape(&id));
                }
            }
            Choice::Layer => {
                for id in select_ids {
                    out.0.append(&mut gcode.0.get_same_z(&id));
                }
                for id in deselect_ids {
                    out.1.append(&mut gcode.0.get_same_z(&id));
                }
            }
        }
        out
    };
    for id in events.0 {
        if let Some(entity) = map.0.get(&id) {
            if let Ok((mut select_me, _)) = selectables.get_mut(*entity) {
                select_me.is_selected = true;
            }
        }
    }
    for id in events.1 {
        if let Some(entity) = map.0.get(&id) {
            if let Ok((mut select_me, _)) = selectables.get_mut(*entity) {
                select_me.is_selected = false;
            }
        }
    }
}
