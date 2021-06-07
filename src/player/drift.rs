use crate::geom::{Quat, Vec3};
use crate::player::{Boost, BoostKind, Physics, Stats, Wheelie};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Drift {
    state: State,
    outside_drift: Option<OutsideDrift>,
}

impl Drift {
    pub fn new(stats: &Stats) -> Drift {
        let outside_drift = if stats.vehicle.drift_kind.is_inside() {
            None
        } else {
            Some(OutsideDrift::new())
        };

        Drift {
            state: State::Idle,
            outside_drift,
        }
    }

    pub fn is_hopping(&self) -> bool {
        match &self.state {
            State::Hop(_) => true,
            _ => false,
        }
    }

    pub fn has_hop_height(&self) -> bool {
        match &self.state {
            State::Hop(hop) => hop.pos_y > 0.0,
            _ => false,
        }
    }

    pub fn hop_dir(&self) -> Option<Vec3> {
        match &self.state {
            State::Hop(hop) => Some(hop.dir),
            _ => None,
        }
    }

    pub fn hop_stick_x(&self) -> Option<f32> {
        match &self.state {
            State::Hop(hop) => hop.stick_x,
            _ => None,
        }
    }

    pub fn is_drifting(&self) -> bool {
        match &self.state {
            State::Drift(_) => true,
            _ => false,
        }
    }

    pub fn drift_stick_x(&self) -> Option<f32> {
        match &self.state {
            State::Drift(drift) => Some(drift.stick_x),
            _ => None,
        }
    }

    pub fn outside_drift_turn_bonus(&self) -> f32 {
        match &self.state {
            State::Drift(drift) => drift.outside_drift_turn_bonus,
            _ => None,
        }
        .unwrap_or(0.0)
    }

    pub fn outside_drift_angle(&self) -> f32 {
        self.outside_drift
            .as_ref()
            .map(|outside_drift| outside_drift.angle)
            .unwrap_or(0.0)
    }

    pub fn update(
        &mut self,
        stats: &Stats,
        drift_input: bool,
        last_drift_input: bool,
        stick_x: f32,
        airtime: u32,
        boost: &mut Boost,
        mut wheelie: Option<&mut Wheelie>,
        physics: &mut Physics,
    ) {
        let ground = airtime == 0;

        if !ground && stick_x != 0.0 {
            match &self.state {
                State::Idle if drift_input => {
                    if let Some(wheelie) = wheelie.as_mut() {
                        wheelie.cancel();
                    }
                    self.state = State::SlipdriftCharge(SlipdriftChargeState::new(stick_x));
                }
                State::SlipdriftCharge(_) if !drift_input => {
                    self.state = State::Idle;
                }
                _ => (),
            }
        }

        match &mut self.state {
            State::Idle if drift_input && !last_drift_input => {
                self.start_hop(wheelie, physics);
            }
            State::SlipdriftCharge(slipdrift_charge) if ground => {
                let stick_x = slipdrift_charge.stick_x;
                self.start_drift(stick_x, stats, physics);
            }
            State::Hop(hop) => {
                hop.update(stick_x);

                if hop.can_start_drift() && ground {
                    if let Some(outside_drift) = &mut self.outside_drift {
                        let hop_stick_x = hop.stick_x.unwrap_or(0.0);
                        outside_drift.update_angle_on_drift_start(hop, hop_stick_x, physics.rot0);
                    }

                    match hop.stick_x {
                        Some(hop_stick_x) if drift_input => {
                            self.start_drift(hop_stick_x, stats, physics);
                        }
                        _ => self.state = State::Idle,
                    }
                } else if !hop.can_start_drift() && drift_input && !last_drift_input {
                    self.start_hop(wheelie, physics);
                }
            }
            State::Drift(_) if airtime > 5 => {
                if let Some(outside_drift) = &mut self.outside_drift {
                    outside_drift.update_angle_while_airborne(physics.rot0);
                }
            }
            _ => (),
        }

        if let Some(outside_drift) = &mut self.outside_drift {
            outside_drift.update_dir(physics.rot0);
        }

        match &mut self.state {
            State::Idle => {
                if let Some(outside_drift) = &mut self.outside_drift {
                    outside_drift.decrease_angle(stats.common.outside_drift_dec);
                }
            }
            State::Drift(drift) => {
                if drift_input {
                    drift.update_outside_drift_turn_bonus();

                    if airtime <= 5 {
                        if let Some(outside_drift) = &mut self.outside_drift {
                            outside_drift.adjust_angle(
                                stats.common.manual_drift_tightness,
                                stats.common.outside_drift_target_angle,
                                drift.stick_x,
                            );
                        }

                        drift.update_mt_charge(stick_x);
                    }
                } else {
                    drift.release_mt(stats.common.mt_duration as u16, boost);

                    self.state = State::Idle;
                }
            }
            _ => (),
        }
    }

    fn start_hop(&mut self, wheelie: Option<&mut Wheelie>, physics: &mut Physics) {
        if let Some(wheelie) = wheelie {
            wheelie.cancel();
        }

        physics.vel0.y = 10.0;
        physics.normal_acceleration = 0.0;

        self.state = State::Hop(HopState::new(physics.rot0));
    }

    fn start_drift(&mut self, hop_stick_x: f32, stats: &Stats, physics: &Physics) {
        let outside_drift_turn_bonus = self.outside_drift.as_ref().map(|_| {
            let speed_ratio = (physics.speed1 / stats.common.base_speed).min(1.0);
            speed_ratio * stats.common.manual_drift_tightness * 0.5
        });
        let is_bike = stats.vehicle.drift_kind.is_bike();
        let drift_state = DriftState::new(hop_stick_x, outside_drift_turn_bonus, is_bike);
        self.state = State::Drift(drift_state);
    }

    pub fn update_hop_physics(&mut self) {
        if let State::Hop(hop) = &mut self.state {
            hop.update_physics();
        }
    }
}

#[derive(Clone, Debug)]
enum State {
    Idle,
    SlipdriftCharge(SlipdriftChargeState),
    Hop(HopState),
    Drift(DriftState),
}

#[derive(Clone, Debug)]
struct SlipdriftChargeState {
    stick_x: f32,
}

impl SlipdriftChargeState {
    pub fn new(stick_x: f32) -> SlipdriftChargeState {
        SlipdriftChargeState {
            stick_x: stick_x.signum(),
        }
    }
}

#[derive(Clone, Debug)]
struct HopState {
    frame: u8,
    dir: Vec3,
    up: Vec3,
    stick_x: Option<f32>,
    pos_y: f32,
    vel_y: f32,
}

impl HopState {
    fn new(rot0: Quat) -> HopState {
        HopState {
            frame: 0,
            dir: rot0.rotate(Vec3::FRONT),
            up: rot0.rotate(Vec3::UP),
            stick_x: None,
            pos_y: 0.0,
            vel_y: 10.0,
        }
    }

    fn update(&mut self, stick_x: f32) {
        self.frame = (self.frame + 1).min(3);

        if self.stick_x.is_none() && stick_x != 0.0 {
            self.stick_x = Some(stick_x.signum() * stick_x.abs().ceil())
        }
    }

    fn can_start_drift(&self) -> bool {
        self.frame >= 3
    }

    fn update_physics(&mut self) {
        let drag_factor = 0.998;
        self.vel_y *= drag_factor;
        let gravity = -1.3;
        self.vel_y += gravity;

        self.pos_y += self.vel_y;

        if self.pos_y < 0.0 {
            self.vel_y = 0.0;
            self.pos_y = 0.0;
        }
    }
}

#[derive(Clone, Debug)]
struct DriftState {
    stick_x: f32,
    outside_drift_turn_bonus: Option<f32>,
    mt_charge: u16,
    smt_charge: Option<u16>,
}

impl DriftState {
    fn new(hop_stick_x: f32, outside_drift_turn_bonus: Option<f32>, is_bike: bool) -> DriftState {
        DriftState {
            stick_x: hop_stick_x,
            outside_drift_turn_bonus,
            mt_charge: 0,
            smt_charge: (!is_bike).then(|| 0),
        }
    }

    fn update_outside_drift_turn_bonus(&mut self) {
        if let Some(turn_bonus) = &mut self.outside_drift_turn_bonus {
            *turn_bonus *= 0.99;
        }
    }

    fn update_mt_charge(&mut self, stick_x: f32) {
        let mt_charge_inc = if stick_x * self.stick_x > 0.4 { 5 } else { 2 };
        self.mt_charge = (self.mt_charge + mt_charge_inc).min(271);
        if let Some(smt_charge) = &mut self.smt_charge {
            if self.mt_charge >= 271 {
                *smt_charge = (*smt_charge + mt_charge_inc).min(301);
            }
        }
    }

    fn release_mt(&self, mt_duration: u16, boost: &mut Boost) {
        match self.smt_charge {
            Some(smt_charge) if smt_charge >= 301 => {
                boost.activate(BoostKind::Weak, 3 * mt_duration);
            }
            _ if self.mt_charge >= 271 => {
                boost.activate(BoostKind::Weak, mt_duration);
            }
            _ => (),
        }
    }
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

    fn update_angle_on_drift_start(&mut self, hop: &HopState, hop_stick_x: f32, rot0: Quat) {
        let front = rot0.rotate(Vec3::FRONT);
        let rej = front.rej_unit(hop.up);
        let sq_norm = rej.sq_norm();
        if sq_norm > f32::EPSILON {
            let rej = rej.normalize();
            let cross = hop.dir.cross(rej);
            let norm = cross.sq_norm().wii_sqrt();
            let dot = hop.dir.dot(rej);
            let angle_diff = norm.wii_atan2(dot).to_degrees();
            let angle_diff = angle_diff * hop_stick_x;
            self.angle = (self.angle + angle_diff).clamp(-60.0, 60.0)
        }
    }

    fn update_angle_while_airborne(&mut self, rot0: Quat) {
        let up = rot0.rotate(Vec3::UP);
        let rej = self.dir.rej_unit(up);
        let sq_norm = rej.sq_norm();
        if sq_norm > f32::EPSILON {
            let rej = rej.normalize();
            let dir = rot0.rotate(Vec3::FRONT);
            let cross = rej.cross(dir);
            let norm = cross.sq_norm().wii_sqrt();
            let dot = rej.dot(dir);
            let angle_diff = norm.wii_atan2(dot).to_degrees();
            let sign = (dir.x * (dir.z - rej.z) - dir.z * (dir.x - rej.x)).signum();
            self.angle += sign * angle_diff;
        }
    }

    fn decrease_angle(&mut self, dec: f32) {
        self.angle = self.angle.signum() * (self.angle.abs() - dec).max(0.0);
    }

    fn adjust_angle(&mut self, manual_drift_tightness: f32, target_angle: f32, drift_stick_x: f32) {
        let last_angle = self.angle * drift_stick_x;
        let next_angle = if last_angle < target_angle {
            (last_angle + 150.0 * manual_drift_tightness).min(target_angle)
        } else if last_angle > target_angle {
            (last_angle - 2.0).max(target_angle)
        } else {
            last_angle
        };
        self.angle = next_angle * drift_stick_x;
    }

    fn update_dir(&mut self, rot0: Quat) {
        self.dir = rot0.rotate(Vec3::FRONT);
    }
}
