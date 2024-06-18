use super::{Choice, GCode, IdMap, PickSelection, Tag, UiResource};
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;

#[derive(Default, Resource, Clone, PartialEq, Hash)]
struct Selection(Vec<Tag>);
/// Update entity selection component state from pointer events.
pub fn update_selections(
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
                for id in gcode.0.get_same_z(&id.id) {
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
                for id in gcode.0.get_same_z(&id.id) {
                    let entity = map.0.get(&id).unwrap();
                    let (mut deselect_me, _) =
                        selectables.get_mut(*entity).expect("entity not found");
                    deselect_me.is_selected = false;
                }
            }
        }
    }
}
