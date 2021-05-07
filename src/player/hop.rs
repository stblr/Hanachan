use crate::geom::Vec3;
use crate::player::Physics;

#[derive(Clone, Debug)]
pub struct Hop {
    inner: Option<Inner>,
}

impl Hop {
    pub fn new() -> Hop {
        Hop { inner: None }
    }

    pub fn is_hopping(&self) -> bool {
        self.inner.is_some()
    }

    pub fn dir(&self) -> Option<Vec3> {
        self.inner.as_ref().map(|inner| inner.dir)
    }

    pub fn stick_x(&self) -> Option<f32> {
        self.inner.as_ref().and_then(|inner| inner.stick_x)
    }

    pub fn update(&mut self, drift: bool, stick_x: f32, physics: &mut Physics) {
        if let Some(inner) = &mut self.inner {
            if inner.stick_x.is_none() && stick_x != 0.0 {
                inner.stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
            }
        } else if drift {
            physics.vel0.y = 10.0;
            physics.normal_acceleration = 0.0;
            self.inner = Some(Inner {
                dir: physics.rot0.rotate(Vec3::FRONT),
                stick_x: None,
            });
        }
    }
}

#[derive(Clone, Debug)]
struct Inner {
    pub dir: Vec3,
    pub stick_x: Option<f32>,
}
