use std::ops::{Add, AddAssign, Mul, Neg, Sub};

use crate::fs::{Error, Parse, SliceRefExt};
use crate::wii::F32Ext;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
    pub const DOWN: Vec3 = Vec3::new(0.0, -1.0, 0.0);
    pub const FRONT: Vec3 = Vec3::new(0.0, 0.0, 1.0);
    pub const BACK: Vec3 = Vec3::new(0.0, 0.0, -1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn proj_unit(self, other: Vec3) -> Vec3 {
        self.dot(other) * other
    }

    pub fn rej_unit(self, other: Vec3) -> Vec3 {
        self - self.proj_unit(other)
    }

    pub fn sq_norm(self) -> f32 {
        self.dot(self)
    }

    pub fn norm(self) -> f32 {
        let sq_norm = self.sq_norm();
        if sq_norm <= f32::EPSILON {
            0.0
        } else {
            sq_norm.wii_sqrt()
        }
    }

    pub fn normalize(self) -> Vec3 {
        let norm = self.norm();
        if norm == 0.0 {
            self
        } else {
            1.0 / norm * self
        }
    }

    pub fn wii_sin(self) -> Vec3 {
        Vec3 {
            x: self.x.wii_sin(),
            y: self.y.wii_sin(),
            z: self.z.wii_sin(),
        }
    }

    pub fn wii_cos(self) -> Vec3 {
        Vec3 {
            x: self.x.wii_cos(),
            y: self.y.wii_cos(),
            z: self.z.wii_cos(),
        }
    }

    pub fn to_radians(self) -> Vec3 {
        Vec3 {
            x: self.x.to_radians(),
            y: self.y.to_radians(),
            z: self.z.to_radians(),
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        *self = *self + other;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self * v.x,
            y: self * v.y,
            z: self * v.z,
        }
    }
}

impl Parse for Vec3 {
    fn parse(input: &mut &[u8]) -> Result<Vec3, Error> {
        Ok(Vec3::new(input.take()?, input.take()?, input.take()?))
    }
}
