use crate::geom::{Mat33, Vec3};
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct Lean {
    rot: f32,
    rot_diff: f32,
}

impl Lean {
    pub fn new() -> Lean {
        Lean {
            rot: 0.0,
            rot_diff: 0.08,
        }
    }

    pub fn rot(&self) -> f32 {
        self.rot
    }

    pub fn update(&mut self, stick_x: f32, physics: &mut Physics) {
        let s = if stick_x.abs() <= 0.2 {
            self.rot *= 0.9;
            0.0
        } else {
            let s = -stick_x.signum();
            self.rot -= s * self.rot_diff;
            s
        };

        if self.rot.abs() > 0.6 {
            self.rot = self.rot.signum() * 0.6;
        } else {
            let right = Mat33::from(physics.mat()) * Vec3::RIGHT;
            physics.vel0 += s * right;
        }

        // TODO handle drift
        physics.rot_vec2.z += 0.05 * self.rot;
    }
}
