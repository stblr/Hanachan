use crate::geom::Vec3;
use crate::player::{Boost, BoostKind, Physics, Wheelie};

#[derive(Clone, Debug)]
pub struct Drift {
    mt_duration: u16,
    state: State,
}

impl Drift {
    pub fn new(mt_duration: u16) -> Drift {
        Drift {
            mt_duration,
            state: State::Idle,
        }
    }

    pub fn is_hopping(&self) -> bool {
        match self.state {
            State::Hop { pos_y, .. } => pos_y > 0.0,
            _ => false,
        }
    }

    pub fn hop_dir(&self) -> Option<Vec3> {
        match self.state {
            State::Hop { dir, .. } => Some(dir),
            _ => None,
        }
    }

    pub fn hop_stick_x(&self) -> Option<f32> {
        match self.state {
            State::Hop { stick_x, .. } => stick_x,
            _ => None,
        }
    }

    pub fn is_drifting(&self) -> bool {
        match self.state {
            State::Drift { .. } => true,
            _ => false,
        }
    }

    pub fn drift_stick_x(&self) -> Option<f32> {
        match self.state {
            State::Drift { stick_x, .. } => Some(stick_x),
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        drift_input: bool,
        stick_x: f32,
        boost: &mut Boost,
        wheelie: Option<&mut Wheelie>,
        physics: &mut Physics,
        ground: bool,
    ) {
        match &mut self.state {
            State::Idle if drift_input => {
                if let Some(wheelie) = wheelie {
                    wheelie.cancel();
                }

                physics.vel0.y = 10.0;
                physics.normal_acceleration = 0.0;

                self.state = State::Hop {
                    frame: 0,
                    dir: physics.rot0.rotate(Vec3::FRONT),
                    stick_x: None,
                    pos_y: 0.0,
                    vel_y: 10.0,
                };
            }
            State::Hop {
                frame,
                stick_x: hop_stick_x,
                ..
            } => {
                *frame = (*frame + 1).min(3);

                if hop_stick_x.is_none() && stick_x != 0.0 {
                    *hop_stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
                }

                if *frame >= 3 && ground {
                    if let Some(hop_stick_x) = hop_stick_x {
                        self.state = State::Drift {
                            stick_x: *hop_stick_x,
                            mt_charge: 0,
                        };
                    }
                }
            }
            _ => (),
        }

        if let State::Drift {
            stick_x: drift_stick_x,
            mt_charge,
        } = &mut self.state
        {
            if drift_input {
                let mt_charge_inc = if stick_x * *drift_stick_x > 0.4 { 5 } else { 2 };
                *mt_charge = (*mt_charge + mt_charge_inc).min(270);
            } else {
                if *mt_charge >= 270 {
                    boost.activate(BoostKind::Weak, self.mt_duration);
                }

                self.state = State::Idle;
            }
        }
    }

    pub fn update_hop_physics(&mut self) {
        if let State::Hop { pos_y, vel_y, .. } = &mut self.state {
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
enum State {
    Idle,
    Hop {
        frame: u8,
        dir: Vec3,
        stick_x: Option<f32>,
        pos_y: f32,
        vel_y: f32,
    },
    Drift {
        stick_x: f32,
        mt_charge: u16,
    },
}
