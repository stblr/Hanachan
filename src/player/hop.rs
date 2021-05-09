use crate::geom::Vec3;
use crate::player::{Physics, Wheelie};

#[derive(Clone, Debug)]
pub struct Hop {
    inner: Option<Inner>,
}

impl Hop {
    pub fn new() -> Hop {
        Hop { inner: None }
    }

    pub fn is_hopping(&self) -> bool {
        self.inner.as_ref().map(|inner| inner.pos_y > 0.0).unwrap_or(false)
    }

    pub fn dir(&self) -> Option<Vec3> {
        self.inner.as_ref().map(|inner| inner.dir)
    }

    pub fn stick_x(&self) -> Option<f32> {
        self.inner.as_ref().and_then(|inner| inner.stick_x)
    }

    pub fn update(
        &mut self,
        drift: bool,
        stick_x: f32,
        wheelie: Option<&mut Wheelie>,
        physics: &mut Physics,
    ) {
        if let Some(inner) = &mut self.inner {
            if inner.stick_x.is_none() && stick_x != 0.0 {
                inner.stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
            }
        } else if drift {
            if let Some(wheelie) = wheelie {
                wheelie.cancel();
            }

            physics.vel0.y = 10.0;
            physics.normal_acceleration = 0.0;

            self.inner = Some(Inner {
                dir: physics.rot0.rotate(Vec3::FRONT),
                stick_x: None,
                pos_y: 0.0,
                vel_y: 10.0,
            });
        }
    }

    pub fn update_physics(&mut self) {
        if let Some(inner) = &mut self.inner {
            let drag_factor = 0.998;
            inner.vel_y *= drag_factor;
            let gravity = -1.3;
            inner.vel_y += gravity;

            inner.pos_y += inner.vel_y;

            if inner.pos_y < 0.0 {
                inner.vel_y = 0.0;
                inner.pos_y = 0.0;
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Inner {
    dir: Vec3,
    stick_x: Option<f32>,
    pos_y: f32,
    vel_y: f32,
}
