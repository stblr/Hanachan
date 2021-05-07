use crate::geom::Vec3;
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct Wheelie {
    frame: u16,
    rot: f32,
    rot_dec: f32,
}

impl Wheelie {
    pub fn new() -> Wheelie {
        Wheelie {
            frame: 0,
            rot: 0.0,
            rot_dec: 0.0,
        }
    }

    pub fn is_wheelieing(&self) -> bool {
        self.frame > 0
    }

    pub fn rot(&self) -> f32 {
        self.rot
    }

    pub fn update(&mut self, base_speed: f32, trick_is_up: bool, physics: &mut Physics) {
        if self.frame > 0 || trick_is_up {
            self.frame += 1;

            if self.should_cancel(base_speed, physics) {
                self.frame = 0;
                self.rot_dec = 0.0;
            } else {
                self.rot = (self.rot + 0.01).min(0.07);
                physics.rot_vec0.x *= 0.9;
            }
        } else if self.rot > 0.0 {
            self.rot_dec += 0.001;
            self.rot = (self.rot - self.rot_dec).max(0.0);
        }

        let cos = Vec3::UP.dot(physics.dir);
        if cos <= 0.5 || self.frame < 15 {
            physics.rot_vec2.x -= self.rot * (1.0 - cos.abs());
        }
    }

    fn should_cancel(&self, base_speed: f32, physics: &Physics) -> bool {
        if self.frame < 15 {
            false
        } else if self.frame > 180 {
            true
        } else {
            let speed1_ratio = physics.speed1 / base_speed;
            physics.speed1 < 0.0 || speed1_ratio < 0.3
        }
    }
}
