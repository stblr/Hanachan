use crate::geom::Vec3;
use crate::player::{Physics, Wheelie};

#[derive(Clone, Debug)]
pub enum Drift {
    Idle,
    Hop {
        dir: Vec3,
        stick_x: Option<f32>,
        pos_y: f32,
        vel_y: f32,
    },
    Drift {
        stick_x: f32,
    },
}

impl Drift {
    pub fn is_hopping(&self) -> bool {
        match self {
            Drift::Hop { pos_y, .. } => *pos_y > 0.0,
            _ => false,
        }
    }

    pub fn is_drifting(&self) -> bool {
        match self {
            Drift::Drift { .. } => true,
            _ => false,
        }
    }

    pub fn hop_dir(&self) -> Option<Vec3> {
        match self {
            Drift::Hop { dir, .. } => Some(*dir),
            _ => None,
        }
    }

    pub fn hop_stick_x(&self) -> Option<f32> {
        match self {
            Drift::Hop { stick_x, .. } => *stick_x,
            _ => None,
        }
    }

    pub fn drift_stick_x(&self) -> Option<f32> {
        match self {
            Drift::Drift { stick_x, .. } => Some(*stick_x),
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        drift: bool,
        stick_x: f32,
        wheelie: Option<&mut Wheelie>,
        physics: &mut Physics,
        ground: bool,
    ) {
        match self {
            Drift::Idle if drift => {
                if let Some(wheelie) = wheelie {
                    wheelie.cancel();
                }

                physics.vel0.y = 10.0;
                physics.normal_acceleration = 0.0;

                *self = Drift::Hop {
                    dir: physics.rot0.rotate(Vec3::FRONT),
                    stick_x: None,
                    pos_y: 0.0,
                    vel_y: 10.0,
                };
            }
            Drift::Hop {
                stick_x: hop_stick_x,
                ..
            } => {
                if hop_stick_x.is_none() && stick_x != 0.0 {
                    *hop_stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
                }
                if ground {
                    if let Some(hop_stick_x) = hop_stick_x {
                        *self = Drift::Drift {
                            stick_x: *hop_stick_x,
                        };
                    }
                }
            }
            _ => (),
        }
    }

    pub fn update_hop_physics(&mut self) {
        if let Drift::Hop { pos_y, vel_y, .. } = self {
            let drag_factor = 0.998;
            *vel_y *= drag_factor;
            let gravity = -1.3;
            *vel_y += gravity;

            *pos_y += *vel_y;

            if *pos_y < 0.0 {
                *vel_y = 0.0;
                *pos_y = 0.0;
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
