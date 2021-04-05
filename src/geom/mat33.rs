use std::ops::Mul;

use crate::geom::{Mat34, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Mat33 {
    pub e00: f32,
    pub e01: f32,
    pub e02: f32,
    pub e10: f32,
    pub e11: f32,
    pub e12: f32,
    pub e20: f32,
    pub e21: f32,
    pub e22: f32,
}

impl From<Mat34> for Mat33 {
    fn from(m: Mat34) -> Mat33 {
        Mat33 {
            e00: m.e00,
            e01: m.e01,
            e02: m.e02,
            e10: m.e10,
            e11: m.e11,
            e12: m.e12,
            e20: m.e20,
            e21: m.e21,
            e22: m.e22,
        }
    }
}

impl Mul<Vec3> for Mat33 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.e00 * v.x + self.e01 * v.y + self.e02 * v.z,
            y: self.e10 * v.x + self.e11 * v.y + self.e12 * v.z,
            z: self.e20 * v.x + self.e21 * v.y + self.e22 * v.z,
        }
    }
}
