use std::ops::Mul;

use crate::geom::{Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Mat34 {
    pub e00: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e10: f32,
    pub e11: f32,
    pub e12: f32,
    pub e13: f32,
    pub e20: f32,
    pub e21: f32,
    pub e22: f32,
    pub e23: f32,
}

impl Mat34 {
    pub fn from_angles_and_pos(angles: Vec3, pos: Vec3) -> Mat34 {
        let sin = angles.wii_sin();
        let cos = angles.wii_cos();

        Mat34 {
            e00: cos.y * cos.z,
            e01: sin.x * sin.y * cos.z - sin.z * cos.x,
            e02: cos.x * cos.z * sin.y + sin.x * sin.z,
            e03: pos.x,
            e10: sin.z * cos.y,
            e11: sin.x * sin.y * sin.z + cos.x * cos.z,
            e12: sin.z * cos.x * sin.y - sin.x * cos.z,
            e13: pos.y,
            e20: -sin.y,
            e21: sin.x * cos.y,
            e22: cos.x * cos.y,
            e23: pos.z,
        }
    }

    pub fn from_quat_and_pos(q: Quat, pos: Vec3) -> Mat34 {
        Mat34 {
            e00: 1.0 - 2.0 * q.y * q.y - 2.0 * q.z * q.z,
            e01: 2.0 * q.x * q.y - 2.0 * q.w * q.z,
            e02: 2.0 * q.x * q.z + 2.0 * q.w * q.y,
            e03: pos.x,
            e10: 2.0 * q.x * q.y + 2.0 * q.w * q.z,
            e11: 1.0 - 2.0 * q.x * q.x - 2.0 * q.z * q.z,
            e12: 2.0 * q.y * q.z - 2.0 * q.w * q.x,
            e13: pos.y,
            e20: 2.0 * q.x * q.z - 2.0 * q.w * q.y,
            e21: 2.0 * q.y * q.z + 2.0 * q.w * q.x,
            e22: 1.0 - 2.0 * q.x * q.x - 2.0 * q.y * q.y,
            e23: pos.z,
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Mat34 {
        let q = Quat::from_axis_angle(axis, angle);
        Mat34::from_quat_and_pos(q, Vec3::ZERO)
    }

    pub fn from_diag(diag: Vec3) -> Mat34 {
        Mat34 {
            e00: diag.x,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e10: 0.0,
            e11: diag.y,
            e12: 0.0,
            e13: 0.0,
            e20: 0.0,
            e21: 0.0,
            e22: diag.z,
            e23: 0.0,
        }
    }

    pub fn transpose(self) -> Mat34 {
        Mat34 {
            e00: self.e00,
            e01: self.e10,
            e02: self.e20,
            e03: 0.0,
            e10: self.e01,
            e11: self.e11,
            e12: self.e21,
            e13: 0.0,
            e20: self.e02,
            e21: self.e12,
            e22: self.e22,
            e23: 0.0,
        }
    }
}

impl Mul for Mat34 {
    type Output = Mat34;

    fn mul(self, other: Mat34) -> Mat34 {
        fn mul_entry(row: [f32; 4], col: [f32; 4]) -> f32 {
            let mut acc = row[0] * col[0];
            acc = (row[1] as f64 * col[1] as f64 + acc as f64) as f32;
            acc = (row[2] as f64 * col[2] as f64 + acc as f64) as f32;
            acc = (row[3] as f64 * col[3] as f64 + acc as f64) as f32;
            acc
        }

        let row0 = [self.e00, self.e01, self.e02, self.e03];
        let row1 = [self.e10, self.e11, self.e12, self.e13];
        let row2 = [self.e20, self.e21, self.e22, self.e23];

        let col0 = [other.e00, other.e10, other.e20, 0.0];
        let col1 = [other.e01, other.e11, other.e21, 0.0];
        let col2 = [other.e02, other.e12, other.e22, 0.0];
        let col3 = [other.e03, other.e13, other.e23, 1.0];

        Mat34 {
            e00: mul_entry(row0, col0),
            e01: mul_entry(row0, col1),
            e02: mul_entry(row0, col2),
            e03: mul_entry(row0, col3),
            e10: mul_entry(row1, col0),
            e11: mul_entry(row1, col1),
            e12: mul_entry(row1, col2),
            e13: mul_entry(row1, col3),
            e20: mul_entry(row2, col0),
            e21: mul_entry(row2, col1),
            e22: mul_entry(row2, col2),
            e23: mul_entry(row2, col3),
        }
    }
}

impl Mul<Vec3> for Mat34 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        fn mul_entry(row: [f32; 4], v: Vec3) -> f32 {
            let mut tmp0 = row[0] * v.x;
            tmp0 = (row[2] as f64 * v.z as f64 + tmp0 as f64) as f32;
            let tmp1 = row[1] * v.y + row[3];
            tmp0 + tmp1
        }

        let row0 = [self.e00, self.e01, self.e02, self.e03];
        let row1 = [self.e10, self.e11, self.e12, self.e13];
        let row2 = [self.e20, self.e21, self.e22, self.e23];

        Vec3 {
            x: mul_entry(row0, v),
            y: mul_entry(row1, v),
            z: mul_entry(row2, v),
        }
    }
}
