use crate::fs::RkgTrick;
use crate::geom::Vec3;
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct Wheelie {
    is_wheelieing: bool,
    cooldown: u16,
    frame: u16,
    rot: f32,
    rot_dec: f32,
}

impl Wheelie {
    pub fn new() -> Wheelie {
        Wheelie {
            is_wheelieing: false,
            cooldown: 0,
            frame: 0,
            rot: 0.0,
            rot_dec: 0.0,
        }
    }

    pub fn is_wheelieing(&self) -> bool {
        self.is_wheelieing
    }

    pub fn rot(&self) -> f32 {
        self.rot
    }

    pub fn update(
        &mut self,
        base_speed: f32,
        trick_input: Option<RkgTrick>,
        is_drifting: bool,
        physics: &mut Physics,
    ) {
        match trick_input {
            Some(RkgTrick::Up) => self.try_start(is_drifting),
            Some(RkgTrick::Down) => self.try_cancel(),
            _ => (),
        }

        self.cooldown = self.cooldown.saturating_sub(1);

        if self.is_wheelieing {
            self.frame += 1;

            if self.should_cancel(base_speed, physics) {
                self.cancel();
            } else {
                self.rot = (self.rot + 0.01).min(0.07);
                physics.rot_vec0.x *= 0.9;
            }
        } else if self.rot > 0.0 {
            self.rot_dec += 0.001;
            self.rot = (self.rot - self.rot_dec).max(0.0);
        }

        let cos = Vec3::UP.dot(physics.vel1_dir);
        if cos <= 0.5 || self.frame < 15 {
            physics.rot_vec2.x -= self.rot * (1.0 - cos.abs());
        }
    }

    fn try_start(&mut self, is_drifting: bool) {
        if !self.is_wheelieing && self.cooldown == 0 && !is_drifting {
            self.is_wheelieing = true;
            self.cooldown = 20;
        }
    }

    fn try_cancel(&mut self) {
        if self.is_wheelieing && self.cooldown == 0 {
            self.cancel();
            self.cooldown = 20;
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

    pub fn cancel(&mut self) {
        self.is_wheelieing = false;
        self.frame = 0;
        self.rot_dec = 0.0;
    }
}
