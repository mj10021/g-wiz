use crate::{models::Pos5, GCode, Id, VertexMap, State};
use bevy::prelude::*;
use std::collections::HashSet;

/// delete the selected node and stitch together the previous and following node
fn merge_delete(
    mut state: ResMut<State>,
) {
    let selection = state.selection
    for id in selection {
        merge_delete_vertex(&mut gcode, &mut vertices, &id);
    }
}
fn merge_delete_vertex(gcode: &mut ResMut<GCode>, vertices: &mut ResMut<VertexMap>, id: &Id) {
    // Remove the g1 line from the gcode model
    for (i, line) in gcode.0.lines.iter().enumerate() {
        if line.id == *id {
            gcode.0.lines.remove(i);
            break;
        }
    }

    let v = vertices.0[id];

    // if there is a previous move, change it to the final position of the deleted node
    if let Some(p) = &v.prev {
        let mut p = vertices.0[&p];
        let flow = Pos5::flow(&v.position, p.position);
        for i in 0..5 {
            p.position[i] = v.position[i] * flow;
        }
        let prev_prev = {
            if let Some(pp) = p.prev {
                let pp = vertices.0[&pp];
                pp.position
            } else {
                [0.0; 5]
            }
        };
        let new_dist = p.position.dist_xyz(prev_prev);
        p.position[3] = new_dist * flow;
    }
    vertices.0.remove(id);
}

fn hole_delete(mut gcode: ResMut<GCode>, mut vertices: ResMut<VertexMap>, id: &Id) {}
