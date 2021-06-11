use crate::player::Collision;

#[derive(Clone, Debug)]
pub struct BoostRamp {
    duration: u16,
}

impl BoostRamp {
    pub fn new() -> BoostRamp {
        BoostRamp {
            duration: 0,
        }
    }

    pub fn enabled(&self) -> bool {
        self.duration > 0
    }

    pub fn try_start<'a>(&mut self, mut collisions: impl Iterator<Item = &'a Collision>) {
        if collisions.any(Collision::has_boost_ramp) {
            self.duration = 60;
        }
    }

    pub fn update(&mut self) {
        self.duration = self.duration.saturating_sub(1);
    }
}
