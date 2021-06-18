use std::ops::{Add, AddAssign, Mul};

use crate::fs::{Error, Parse, SliceRefExt};
use crate::geom::Vec3;
use crate::wii::F32Ext;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat::new(0.0, 0.0, 0.0, 1.0);
    pub const BACK: Quat = Quat::new(0.0, 1.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Quat {
        Quat { x, y, z, w }
    }

    pub fn from_angles(angles: Vec3) -> Quat {
        let sinh = (0.5 * angles).wii_sin();
        let cosh = (0.5 * angles).wii_cos();

        Quat {
            x: cosh.z * cosh.y * sinh.x - sinh.z * sinh.y * cosh.x,
            y: cosh.z * sinh.y * cosh.x + sinh.z * cosh.y * sinh.x,
            z: sinh.z * cosh.y * cosh.x - cosh.z * sinh.y * sinh.x,
            w: cosh.z * cosh.y * cosh.x + sinh.z * sinh.y * sinh.x,
        }
    }

    pub fn from_vecs(from: Vec3, to: Vec3) -> Quat {
        let s = (2.0 * (from.dot(to) + 1.0)).wii_sqrt();
        if s <= f32::EPSILON {
            return Quat::IDENTITY;
        }
        let recip = 1.0 / s;
        let cross = from.cross(to);
        Quat {
            x: recip * cross.x,
            y: recip * cross.y,
            z: recip * cross.z,
            w: 0.5 * s,
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Quat {
        let half = 0.5 * angle;
        Quat {
            x: half.wii_sin() * axis.x,
            y: half.wii_sin() * axis.y,
            z: half.wii_sin() * axis.z,
            w: half.wii_cos(),
        }
    }

    pub fn invert(self) -> Quat {
        Quat {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn dot(self, other: Quat) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn sq_norm(self) -> f32 {
        self.dot(self)
    }

    pub fn normalize(self) -> Quat {
        let sq_norm = self.sq_norm();
        if sq_norm <= f32::EPSILON {
            self
        } else {
            let norm = sq_norm.wii_sqrt();
            1.0 / norm * self
        }
    }

    pub fn rotate(self, v: Vec3) -> Vec3 {
        Vec3::from(self * v * self.invert())
    }

    pub fn inv_rotate(self, v: Vec3) -> Vec3 {
        Vec3::from(self.invert() * v * self)
    }

    pub fn slerp_to(self, other: Quat, t: f32) -> Quat {
        let dot = self.dot(other).clamp(-1.0, 1.0);
        let angle = (dot.abs() as f64).acos() as f32; // TODO wii_acos?
        let sin = angle.wii_sin();

        let (s, t) = if sin.abs() >= 1e-5 {
            let recip = 1.0 / sin;
            let s = recip * (angle - t * angle).wii_sin();
            let t = recip * (t * angle).wii_sin();
            (s, t)
        } else {
            (1.0 - t, t)
        };

        let t = dot.signum() * t;

        s * self + t * other
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

impl Mul<Vec3> for Quat {
    type Output = Quat;

    fn mul(self, v: Vec3) -> Quat {
        Quat {
            x: self.y * v.z - self.z * v.y + self.w * v.x,
            y: self.z * v.x - self.x * v.z + self.w * v.y,
            z: self.x * v.y - self.y * v.x + self.w * v.z,
            w: -(self.x * v.x + self.y * v.y + self.z * v.z),
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

impl Parse for Quat {
    fn parse(input: &mut &[u8]) -> Result<Quat, Error> {
        Ok(Quat::new(
            input.take()?,
            input.take()?,
            input.take()?,
            input.take()?,
        ))
    }
}
