use crate::geom::Vec3;
use crate::player::{Floor, Physics};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Dive {
    rot: f32,
}

impl Dive {
    pub fn new() -> Dive {
        Dive { rot: 0.0 }
    }

    pub fn update(
        &mut self,
        stick_y: f32,
        floor: &Floor,
        has_rot_bonus: bool,
        physics: &mut Physics,
    ) {
        self.rot *= 0.96;

        if !floor.is_airborne() {
            return;
        }

        let mut rot_diff = stick_y;

        if has_rot_bonus {
            rot_diff = (rot_diff + 0.4).min(1.0);
        }

        if floor.airtime() <= 50 {
            rot_diff *= floor.airtime() as f32 / 50.0
        } else if rot_diff.abs() < 0.1 {
            self.rot -= 0.05 * (self.rot + 0.025);
        }

        self.rot = (self.rot + 0.005 * rot_diff).clamp(-0.8, 0.8);

        physics.rot_vec2.x += self.rot;

        if floor.airtime() >= 50 {
            let up = physics.rot0.rotate(Vec3::UP);
            let cross = physics.up.cross(up);
            let norm = cross.sq_norm().wii_sqrt();
            let dot = physics.up.dot(up);
            let angle = norm.wii_atan2(dot).abs().to_degrees();
            let angle = angle - 20.0;

            if angle > 0.0 {
                let s = (angle / 20.0).min(1.0);
                if physics.rot0.rotate(Vec3::FRONT).y <= 0.0 {
                    physics.gravity *= 1.0 + 0.2 * s;
                } else {
                    physics.gravity *= 1.0 - 0.2 * s;
                }
            }
        }
    }
}
