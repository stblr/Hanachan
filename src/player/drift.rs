use crate::geom::Vec3;
use crate::player::{Boost, BoostKind, Physics, Stats, Wheelie};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Drift {
    is_bike: bool,
    base_speed: f32,
    manual_drift_tightness: f32,
    outside_drift_target_angle: f32,
    outside_drift_dec: f32,
    mt_duration: u16,
    state: State,
    outside_drift: Option<OutsideDrift>,
}

impl Drift {
    pub fn new(stats: &Stats) -> Drift {
        let outside_drift = if stats.vehicle.kind.is_inside_drift() {
            None
        } else {
            Some(OutsideDrift::new())
        };

        Drift {
            is_bike: stats.vehicle.kind.is_bike(),
            base_speed: stats.common.base_speed,
            manual_drift_tightness: stats.common.manual_drift_tightness,
            outside_drift_target_angle: stats.common.outside_drift_target_angle,
            outside_drift_dec: stats.common.outside_drift_dec,
            mt_duration: stats.common.mt_duration as u16,
            state: State::Idle,
            outside_drift,
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

    pub fn outside_drift_turn_bonus(&self) -> f32 {
        match self.state {
            State::Drift {
                outside_drift_turn_bonus: Some(outside_drift_turn_bonus),
                ..
            } => outside_drift_turn_bonus,
            _ => 0.0,
        }
    }

    pub fn outside_drift_angle(&self) -> f32 {
        self.outside_drift.as_ref().map(|outside_drift| outside_drift.angle).unwrap_or(0.0)
    }

    pub fn update(
        &mut self,
        drift_input: bool,
        stick_x: f32,
        airtime: u32,
        boost: &mut Boost,
        wheelie: Option<&mut Wheelie>,
        physics: &mut Physics,
    ) {
        let ground = airtime == 0;

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
                    up: physics.rot0.rotate(Vec3::UP),
                    stick_x: None,
                    pos_y: 0.0,
                    vel_y: 10.0,
                };
            }
            State::Hop {
                frame,
                dir: hop_dir,
                up: hop_up,
                stick_x: hop_stick_x,
                ..
            } => {
                *frame = (*frame + 1).min(3);

                if hop_stick_x.is_none() && stick_x != 0.0 {
                    *hop_stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
                }

                if *frame >= 3 && ground {
                    if let Some(hop_stick_x) = hop_stick_x {
                        let outside_drift_turn_bonus = match &mut self.outside_drift {
                            Some(outside_drift) => {
                                let front = physics.rot0.rotate(Vec3::FRONT);
                                let rej = front.rej_unit(*hop_up);
                                let sq_norm = rej.sq_norm();
                                if sq_norm > f32::EPSILON {
                                    let rej = rej.normalize();
                                    let cross = hop_dir.cross(rej);
                                    let norm = cross.sq_norm().wii_sqrt();
                                    let dot = hop_dir.dot(rej);
                                    let angle_diff = norm.wii_atan2(dot).to_degrees();
                                    let angle_diff = angle_diff * *hop_stick_x;
                                    let angle = &mut outside_drift.angle;
                                    *angle = (*angle + angle_diff).clamp(-60.0, 60.0)
                                }

                                let speed_ratio = (physics.speed1 / self.base_speed).min(1.0);
                                Some(speed_ratio * self.manual_drift_tightness * 0.5)
                            }
                            None => None,
                        };
                        self.state = State::Drift {
                            stick_x: *hop_stick_x,
                            outside_drift_turn_bonus,
                            mt_charge: 0,
                            smt_charge: (!self.is_bike).then(|| 0),
                        };
                    }
                }
            }
            State::Drift { .. } if !ground => {
                if let Some(outside_drift) = &mut self.outside_drift {
                    let up = physics.rot0.rotate(Vec3::UP);
                    let rej = outside_drift.dir.rej_unit(up);
                    let sq_norm = rej.sq_norm();
                    if sq_norm > f32::EPSILON {
                        let rej = rej.normalize();
                        let dir = physics.rot0.rotate(Vec3::FRONT);
                        let cross = rej.cross(dir);
                        let norm = cross.sq_norm().wii_sqrt();
                        let dot = rej.dot(dir);
                        let angle_diff = norm.wii_atan2(dot).to_degrees();
                        let sign = (dir.x * (dir.z - rej.z) - dir.z * (dir.x - rej.x)).signum();
                        outside_drift.angle += sign * angle_diff;
                    }
                }
            }
            _ => (),
        }

        if let Some(outside_drift) = &mut self.outside_drift {
            outside_drift.dir = physics.rot0.rotate(Vec3::FRONT);
        }

        match &mut self.state {
            State::Idle => {
                if let Some(outside_drift) = &mut self.outside_drift {
                    let dec = self.outside_drift_dec;
                    let angle = &mut outside_drift.angle;
                    *angle = angle.signum() * (angle.abs() - dec).max(0.0);
                }
            }
            State::Drift {
                stick_x: drift_stick_x,
                outside_drift_turn_bonus,
                mt_charge,
                smt_charge,
            } => {
                if drift_input {
                    if let Some(turn_bonus) = outside_drift_turn_bonus {
                        *turn_bonus *= 0.99;
                    }

                    if airtime <= 5 {
                        if let Some(outside_drift) = &mut self.outside_drift {
                            let last_angle = outside_drift.angle * *drift_stick_x;
                            let target_angle = self.outside_drift_target_angle;
                            let next_angle = if last_angle < target_angle {
                                (last_angle + 150.0 * self.manual_drift_tightness).min(target_angle)
                            } else if last_angle > target_angle {
                                (last_angle - 2.0).max(target_angle)
                            } else {
                                last_angle
                            };
                            outside_drift.angle = next_angle * *drift_stick_x;
                        }

                        let mt_charge_inc = if stick_x * *drift_stick_x > 0.4 { 5 } else { 2 };
                        *mt_charge = (*mt_charge + mt_charge_inc).min(271);
                        if let Some(smt_charge) = smt_charge {
                            if *mt_charge >= 271 {
                                *smt_charge = (*smt_charge + mt_charge_inc).min(301);
                            }
                        }
                    }
                } else {
                    match smt_charge {
                        Some(smt_charge) if *smt_charge >= 301 => {
                            boost.activate(BoostKind::Weak, 3 * self.mt_duration);
                        }
                        _ if *mt_charge >= 271 => {
                            boost.activate(BoostKind::Weak, self.mt_duration);
                        }
                        _ => (),
                    }

                    self.state = State::Idle;
                }
            }
            _ => (),
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
        up: Vec3,
        stick_x: Option<f32>,
        pos_y: f32,
        vel_y: f32,
    },
    Drift {
        stick_x: f32,
        outside_drift_turn_bonus: Option<f32>,
        mt_charge: u16,
        smt_charge: Option<u16>,
    },
}

#[derive(Clone, Debug)]
struct OutsideDrift {
    angle: f32,
    dir: Vec3,
}

impl OutsideDrift {
    fn new() -> OutsideDrift {
        OutsideDrift {
            angle: 0.0,
            dir: Vec3::BACK,
        }
    }
}
