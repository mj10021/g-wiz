use super::{Parsed, Id};
use bevy::math::Vec3;
use core::f32::consts::PI;
trait Transform {
    fn translate(parsed: &mut Parsed, vertex: &Id);
    fn rotate(parsed: &mut Parsed, vertex: &Id);
    fn scale(parsed: &mut Parsed, vertex: &Id);
}

impl Parsed {
    pub fn rotate(&mut self, vertex: &Id, origin: Vec3, angle_x: f32, angle_y: f32, angle_z: f32) {
        let v = self.vertices.get_mut(vertex).unwrap();
        // Translate point back to origin
        let mut x = v.to.x - origin.x;
        let mut y = v.to.y - origin.y;
        let mut z = v.to.z - origin.z;

        // Convert angles from degrees to radians
        let angle_x = angle_x * PI / 180.0;
        let angle_y = angle_y * PI / 180.0;
        let angle_z = angle_z * PI / 180.0;

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
    fn rotate_vertices(&mut self, vertices:Vec<&Id>, theta: f32) {

    }
    fn scale(&mut self, vertex: &Id, origin: Vec3, scale: f32) {
        let v = self.vertices.get_mut(vertex).unwrap();
        v.to.x = origin.x + (v.to.x - origin.x) * scale;
        v.to.y = origin.y + (v.to.y - origin.y) * scale;
        v.to.z = origin.z + (v.to.z - origin.z) * scale;
    }
}