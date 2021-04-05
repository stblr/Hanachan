use std::ops::{Add, AddAssign, Mul};

use crate::geom::Vec3;
use crate::take::{self, Take, TakeFromSlice};
use crate::wii::F32Ext;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const BACK: Quat = Quat::new(0.0, 1.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat { x, y, z, w }
    }

    pub fn invert(self) -> Quat {
        Quat {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn normalize(self) -> Quat {
        let sq_norm = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;
        if sq_norm <= f32::EPSILON {
            self
        } else {
            let norm = sq_norm.wii_sqrt();
            1.0 / norm * self
        }
    }

    pub fn rotate(self, v: Vec3) -> Vec3 {
        Vec3::from(self * Quat::from(v) * self.invert())
    }

    pub fn inv_rotate(self, v: Vec3) -> Vec3 {
        Vec3::from(self.invert() * Quat::from(v) * self)
    }
}

impl Add for Quat {
    type Output = Quat;

    fn add(self, other: Quat) -> Quat {
        Quat {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl AddAssign for Quat {
    fn add_assign(&mut self, other: Quat) {
        *self = *self + other;
    }
}

impl Mul for Quat {
    type Output = Quat;

    fn mul(self, other: Quat) -> Quat {
        Quat {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y + self.y * other.w + self.z * other.x - self.x * other.z,
            z: self.w * other.z + self.z * other.w + self.x * other.y - self.y * other.x,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }
}

impl Mul<Quat> for f32 {
    type Output = Quat;

    fn mul(self, q: Quat) -> Quat {
        Quat {
            x: self * q.x,
            y: self * q.y,
            z: self * q.z,
            w: self * q.w,
        }
    }
}

impl From<Vec3> for Quat {
    fn from(v: Vec3) -> Quat {
        Quat::new(v.x, v.y, v.z, 0.0)
    }
}

impl From<Quat> for Vec3 {
    fn from(q: Quat) -> Vec3 {
        Vec3::new(q.x, q.y, q.z)
    }
}

impl TakeFromSlice for Quat {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Quat, take::Error> {
        Ok(Quat::new(
            slice.take()?,
            slice.take()?,
            slice.take()?,
            slice.take()?,
        ))
    }
}
