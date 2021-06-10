use crate::fs::KclJumpPadVariant;
use crate::player::{Collision, Physics};

#[derive(Clone, Debug)]
pub struct JumpPad {
    variant: Option<KclJumpPadVariant>,
}

impl JumpPad {
    pub fn new() -> JumpPad {
        JumpPad { variant: None }
    }

    pub fn enabled(&self) -> bool {
        self.variant.is_some()
    }

    pub fn speed(&self) -> Option<f32> {
        self.variant.as_ref().map(KclJumpPadVariant::speed)
    }

    pub fn try_start<'a>(
        &mut self,
        physics: &mut Physics,
        collisions: impl Iterator<Item = &'a Collision>,
    ) {
        if self.variant.is_some() {
            return;
        }

        let variant = match collisions.filter_map(Collision::jump_pad).last() {
            Some(variant) => variant,
            None => return,
        };

        physics.vel0.y = variant.vel_y();
        physics.normal_acceleration = 0.0;

        let prev_dir = physics.dir;
        physics.dir.y = 0.0;
        physics.dir = physics.dir.normalize();
        physics.vel1_dir = physics.dir;
        physics.speed1 *= physics.dir.dot(prev_dir);

        physics.speed1 = physics.speed1.max(variant.speed());

        self.variant = Some(variant);
    }

    pub fn end(&mut self) {
        self.variant = None;
    }
}
