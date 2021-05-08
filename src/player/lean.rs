use crate::geom::{Mat33, Vec3};
use crate::player::Physics;
use crate::race::{Race, Stage};

#[derive(Clone, Debug)]
pub struct Lean {
    rot: f32,
    rot_diff: f32,
    rot_cap: f32,
}

impl Lean {
    pub fn new() -> Lean {
        Lean {
            rot: 0.0,
            rot_diff: 0.08,
            rot_cap: 0.6,
        }
    }

    pub fn rot(&self) -> f32 {
        self.rot
    }

    pub fn update(
        &mut self,
        stick_x: f32,
        is_wheelieing: bool,
        physics: &mut Physics,
        race: &Race,
    ) {
        if race.stage() == Stage::Race {
            self.rot_diff += 0.3 * (0.1 - self.rot_diff);
            self.rot_cap += 0.3 * (1.0 - self.rot_cap);
        }

        let s = if stick_x.abs() <= 0.2 || is_wheelieing {
            self.rot *= 0.9;
            0.0
        } else {
            let s = -stick_x.signum();
            self.rot -= s * self.rot_diff;
            s
        };

        if self.rot.abs() > self.rot_cap {
            self.rot = self.rot.signum() * self.rot_cap;
        } else {
            let right = Mat33::from(physics.mat()) * Vec3::RIGHT;
            physics.vel0 += s * right;
        }

        // TODO handle drift
        physics.rot_vec2.z += 0.05 * self.rot;
    }
}
