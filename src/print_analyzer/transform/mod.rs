use super::{Id, Parsed};
use bevy::math::Vec3;
use core::f32::consts::PI;

impl Parsed {
    pub fn translate(&mut self, id: &Id, vec: &Vec3) {
        let (dx, dy, dz) = (vec.x, vec.y, vec.z);
        let Some(v) = self.vertices.get(id) else {
            return;
        }; // in case a non-vertex instruction is searched, do nothing
        if self.dist_from_prev(&v.id) < f32::EPSILON {
            return; // dont translate moves without travel
        }
        let prev = v.prev.unwrap();
        let init_dist = self.dist_from_prev(id);
        let init_flow = self.vertices.get(id).unwrap().to.e;
        let prev_dist = self.dist_from_prev(&prev);
        {
            let pv = self.vertices.get_mut(&prev).unwrap();
            pv.to.x += dx;
            pv.to.y += dy;
            pv.to.z += dz;
        }

        let new_prev_dist = self.dist_from_prev(&prev);

        let prev = self.vertices.get_mut(&prev).unwrap();

        let mut scale = new_prev_dist / prev_dist;
        if scale.is_infinite() || scale.is_nan() {
            scale = 0.0;
        }
        prev.to.e *= scale;

        let new_dist = self.dist_from_prev(id);
        let mut scale = new_dist / init_dist;
        if scale.is_infinite() || scale.is_nan() {
            scale = 0.0;
        }
        let v = self.vertices.get_mut(id).unwrap();
        v.to.e = init_flow * scale;
    }
    pub fn rotate(&mut self, vertex: &Id, origin: &Vec3, angle: &Vec3) {
        let v = self.vertices.get_mut(vertex).unwrap();
        // Translate point back to origin
        let mut x = v.to.x - origin.x;
        let mut y = v.to.y - origin.y;
        let mut z = v.to.z - origin.z;

        // Convert angles from degrees to radians
        let angle_x = angle.x * PI / 180.0;
        let angle_y = angle.y * PI / 180.0;
        let angle_z = angle.z * PI / 180.0;

        // Rotation around X-axis
        let new_y = y * angle_x.cos() - z * angle_x.sin();
        let new_z = y * angle_x.sin() + z * angle_x.cos();
        y = new_y;
        z = new_z;

        // Rotation around Y-axis
        let new_x = x * angle_y.cos() + z * angle_y.sin();
        let new_z = -x * angle_y.sin() + z * angle_y.cos();
        x = new_x;
        z = new_z;

        // Rotation around Z-axis
        let new_x = x * angle_z.cos() - y * angle_z.sin();
        let new_y = x * angle_z.sin() + y * angle_z.cos();
        x = new_x;
        y = new_y;

        // Translate point back
        v.to.x = x + origin.x;
        v.to.y = y + origin.y;
        v.to.z = z + origin.z;
    }
    pub fn scale(&mut self, vertex: &Id, origin: &Vec3, scale: &Vec3) {
        let v = self.vertices.get_mut(vertex).unwrap();
        v.to.x = origin.x + (v.to.x - origin.x) * scale.x;
        v.to.y = origin.y + (v.to.y - origin.y) * scale.y;
        v.to.z = origin.z + (v.to.z - origin.z) * scale.z;
    }
}
