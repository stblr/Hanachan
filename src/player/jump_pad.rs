use crate::fs::KclJumpPadVariant;
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct JumpPad {
    applied_dir: bool,
    variant: Option<KclJumpPadVariant>,
}

impl JumpPad {
    pub fn new() -> JumpPad {
        JumpPad { applied_dir: false, variant: None }
    }

    pub fn applied_dir(&self) -> bool {
        self.applied_dir
    }

    pub fn enabled(&self) -> bool {
        self.variant.is_some()
    }

    pub fn speed(&self) -> Option<f32> {
        self.variant.as_ref().map(KclJumpPadVariant::speed)
    }

    pub fn try_start<'a>(&mut self, physics: &mut Physics, variant: Option<KclJumpPadVariant>) {
        self.applied_dir = false;

        if self.variant.is_some() {
            return;
        }

        let variant = match variant {
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

        self.applied_dir = true;

        self.variant = Some(variant);
    }

    pub fn end(&mut self) {
        self.variant = None;
    }
}
