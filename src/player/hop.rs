use crate::geom::Vec3;
use crate::player::Physics;

#[derive(Clone, Copy, Debug)]
pub struct Hop {
    pub dir: Option<Vec3>,
}

impl Hop {
    pub fn new() -> Hop {
        Hop { dir: None }
    }

    pub fn is_hopping(&self) -> bool {
        self.dir.is_some()
    }

    pub fn update(&mut self, drift: bool, physics: &mut Physics) {
        if !self.is_hopping() && drift {
            physics.vel0.y = 10.0;
            physics.normal_acceleration = 0.0;
            self.dir = Some(physics.rot0.rotate(Vec3::FRONT));
        }
    }
}
