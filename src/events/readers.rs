use crate::{Choice, GCode, IdMap, Tag, UiResource};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
pub fn selection_reader(
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
