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
        drift_stick_x: Option<f32>,
        is_wheelieing: bool,
        physics: &mut Physics,
        race: &Race,
    ) {
        if race.stage() == Stage::Race {
            self.rot_diff += 0.3 * (0.1 - self.rot_diff);
            self.rot_cap += 0.3 * (1.0 - self.rot_cap);
        }

        let (rot_min, rot_max, s) = match drift_stick_x {
            Some(drift_stick_x) => {
                if stick_x == 0.0 {
                    self.rot += 0.05 * (0.5 * drift_stick_x - self.rot);
                } else {
                    self.rot += 0.05 * stick_x
                }
                let (rot_min, rot_max) = if drift_stick_x < 0.0 {
                    (-1.5, -0.7)
                } else {
                    (0.7, 1.5)
                };
                (rot_min, rot_max, -stick_x)
            }
            None => {
                let s = if stick_x.abs() <= 0.2 || is_wheelieing {
                    self.rot *= 0.9;
                    0.0
                } else {
                    let s = -stick_x.signum();
                    self.rot -= s * self.rot_diff;
                    s
                };
                (-self.rot_cap, self.rot_cap, s)
            }
        };

        if self.rot < rot_min {
            self.rot = rot_min;
        } else if self.rot > rot_max {
            self.rot = rot_max;
        } else {
            let right = Mat33::from(physics.mat()) * Vec3::RIGHT;
            physics.vel0 += s * right;
        }

        let is_drifting = drift_stick_x.is_some();
        let drift_factor = if is_drifting { 1.3 } else { 1.0 };

        physics.rot_vec2.z += 0.05 * drift_factor * self.rot;
    }
}
