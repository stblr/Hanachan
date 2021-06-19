use crate::fs::{KclBoostRampVariant, RkgTrick};
use crate::geom::{Mat34, Quat, Vec3};
use crate::player::{Boost, BoostKind, Floor, Physics, Stats, WeightClass, Wheelie};
use crate::wii::F32Ext;

#[derive(Clone, Debug)]
pub struct Trick {
    next_input: RkgTrick,
    next_timer: u8,
    boost_ramp_enabled: bool,
    has_diving_rot_bonus: bool,
    state: State,
}

impl Trick {
    pub fn new() -> Trick {
        Trick {
            next_input: RkgTrick::Up,
            next_timer: 0,
            boost_ramp_enabled: false,
            has_diving_rot_bonus: false,
            state: State::Idle,
        }
    }

    pub fn has_diving_rot_bonus(&self) -> bool {
        self.has_diving_rot_bonus
    }

    pub fn is_tricking(&self) -> bool {
        self.state.is_started()
    }

    pub fn update_next(
        &mut self,
        input: Option<RkgTrick>,
        floor: &Floor,
        boost_ramp_enabled: bool,
        has_boost_ramp: bool,
    ) {
        if let (State::Idle, Some(input)) = (&self.state, input) {
            self.next_input = input;
            self.next_timer = 15;
        }

        if self.is_ready(floor, boost_ramp_enabled) {
            if floor.airtime() >= 3 {
                self.state = State::Ready;
            }

            if boost_ramp_enabled {
                self.boost_ramp_enabled = true;
            }
        } else {
            self.next_timer = self.next_timer.saturating_sub(1);
        }

        if !floor.is_airborne() && !has_boost_ramp {
            self.boost_ramp_enabled = false;
        }
    }

    fn is_ready(&self, floor: &Floor, boost_ramp_enabled: bool) -> bool {
        if self.next_timer == 0 {
            return false;
        }

        if !self.state.is_idle() {
            return false;
        }

        if floor.airtime() == 0 || floor.airtime() > 10 {
            return false;
        }

        floor.has_trickable() || boost_ramp_enabled
    }

    pub fn try_start(
        &mut self,
        stats: &Stats,
        jump_pad_enabled: bool,
        physics: &mut Physics,
        boost_ramp: Option<KclBoostRampVariant>,
        wheelie: Option<&mut Wheelie>,
    ) {
        if !self.state.is_ready() {
            return;
        }

        let speed_ratio = physics.speed1 / stats.common.base_speed;
        if speed_ratio <= 0.5 {
            return;
        }

        let started = Started::new(self.next_input, stats.vehicle.drift_kind.is_bike(), boost_ramp);

        match started.kind {
            Kind::Stunt if started.rot_dir != 0.0 => self.has_diving_rot_bonus = true,
            Kind::Stunt => (), // The game doesn't reset the bonus, which is a bug
            _ => self.has_diving_rot_bonus = false,
        }

        if !jump_pad_enabled {
            started.kind.set_dir_angle(stats.vehicle.weight_class, physics);
        }

        if let Some(wheelie) = wheelie {
            wheelie.cancel();
        }

        self.state = State::Started(started);
    }

    pub fn update_rot(&mut self, physics: &mut Physics) {
        if let State::Started(started) = &mut self.state {
            started.cooldown = started.cooldown.saturating_sub(1);

            started.update_rot();

            physics.non_conserved_special_rot = physics.non_conserved_special_rot * started.rot;
        }
    }

    pub fn try_end(&mut self, is_bike: bool, boost: &mut Boost, physics: &mut Physics) {
        let started = match &self.state {
            State::Started(started) => started,
            _ => return,
        };

        if started.cooldown > 0 {
            return;
        }

        physics.conserved_special_rot = physics.conserved_special_rot * started.rot;

        boost.activate(BoostKind::Medium, started.kind.boost_duration(is_bike));

        self.state = State::Idle;

        self.boost_ramp_enabled = false;
    }
}

#[derive(Clone, Debug)]
enum State {
    Idle,
    Ready,
    Started(Started),
}

impl State {
    pub fn is_idle(&self) -> bool {
        match self {
            State::Idle => true,
            _ => false,
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            State::Ready => true,
            _ => false,
        }
    }

    pub fn is_started(&self) -> bool {
        match self {
            State::Started { .. } => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
struct Started {
    kind: Kind,
    angle: f32,
    angle_diff: f32,
    angle_diff_mul: f32,
    rot_dir: f32,
    rot: Quat,
    cooldown: u8,
}

impl Started {
    fn new(input: RkgTrick, is_bike: bool, boost_ramp: Option<KclBoostRampVariant>) -> Started {
        match boost_ramp.map(|boost_ramp| boost_ramp.id()) {
            Some(0) => Started::new_flip(input, is_bike, true),
            Some(1) => Started::new_flip(input, is_bike, false),
            _ => Started::new_stunt(input, is_bike),
        }
    }

    fn new_stunt(input: RkgTrick, is_bike: bool) -> Started {
        let kind = Kind::Stunt;

        let rot_dir = match input {
            RkgTrick::Left if is_bike => 1.0,
            RkgTrick::Right if is_bike => -1.0,
            _ => 0.0,
        };

        Started::new_inner(kind, rot_dir)
    }

    fn new_flip(input: RkgTrick, is_bike: bool, is_double: bool) -> Started {
        let axis = match input {
            RkgTrick::Up | RkgTrick::Down if is_bike => Axis::X,
            RkgTrick::Up | RkgTrick::Down => Axis::Z,
            RkgTrick::Left | RkgTrick::Right => Axis::Y,
        };
        let kind = Kind::Flip { is_double, axis };

        let rot_dir = match input {
            RkgTrick::Down | RkgTrick::Left => 1.0,
            RkgTrick::Up | RkgTrick::Right => -1.0,
        };

        Started::new_inner(kind, rot_dir)
    }

    fn new_inner(kind: Kind, rot_dir: f32) -> Started {
        let angle_diff = kind.initial_angle_diff();

        Started {
            kind,
            angle: 0.0,
            angle_diff,
            angle_diff_mul: 1.0,
            rot_dir,
            rot: Quat::IDENTITY,
            cooldown: 5,
        }
    }

    fn update_rot(&mut self) {
        self.angle_diff *= self.angle_diff_mul;
        self.angle_diff = self.angle_diff.max(self.kind.min_angle_diff());

        self.angle_diff_mul -= self.kind.angle_diff_mul_dec();
        self.angle_diff_mul = self.angle_diff_mul.max(self.kind.min_angle_diff_mul());

        self.angle += self.angle_diff;
        self.angle = self.angle.min(self.kind.max_angle());

        self.rot = match &self.kind {
            Kind::Stunt => {
                if self.rot_dir == 0.0 {
                    Quat::IDENTITY
                } else {
                    let rot_dir = self.rot_dir;
                    let a = 20.0_f32.to_radians();
                    let b = 60.0_f32.to_radians();
                    let step = 256.0 / 360.0;
                    let sin = (step * self.angle).wii_sin_inner();
                    let angles = Vec3::new(-a * sin, rot_dir * -b * sin, rot_dir * a * sin);
                    Quat::from_angles(angles)
                }
            }
            Kind::Flip { axis, .. } => axis.rot(self.rot_dir * self.angle.to_radians()),
        };
    }
}

#[derive(Clone, Debug)]
enum Kind {
    Stunt,
    Flip { is_double: bool, axis: Axis },
}

impl Kind {
    fn set_dir_angle(&self, weight_class: WeightClass, physics: &mut Physics) {
        let cross = physics.vel1_dir.cross(Vec3::UP);
        let norm = cross.sq_norm().wii_sqrt();
        let dot = physics.vel1_dir.dot(Vec3::UP);
        let angle = norm.wii_atan2(dot).abs().to_degrees();
        let angle = 90.0 - angle;

        let dir_angle = self.dir_angle(weight_class);
        let max_dir_angle_diff = self.max_dir_angle_diff(weight_class);
        if angle > dir_angle {
            return;
        }

        let angle_diff = if dir_angle < angle + max_dir_angle_diff {
            dir_angle - angle
        } else {
            max_dir_angle_diff
        };
        let left = physics.smoothed_up.cross(physics.dir);
        let mat = Mat34::from_axis_angle(left, -angle_diff.to_radians());
        physics.dir = mat * physics.dir;
        physics.vel1_dir = physics.dir;
    }

    fn dir_angle(&self, weight_class: WeightClass) -> f32 {
        match (self, weight_class) {
            (Kind::Stunt { .. }, WeightClass::Light) => 40.0,
            (Kind::Stunt { .. }, WeightClass::Medium) => 36.0,
            (Kind::Stunt { .. }, WeightClass::Heavy) => 32.0,
            (Kind::Flip { .. }, WeightClass::Light) => 45.0,
            (Kind::Flip { .. }, WeightClass::Medium) => 42.0,
            (Kind::Flip { .. }, WeightClass::Heavy) => 39.0,
        }
    }

    fn max_dir_angle_diff(&self, weight_class: WeightClass) -> f32 {
        match (self, weight_class) {
            (Kind::Stunt { .. }, WeightClass::Light) => 15.0,
            (Kind::Stunt { .. }, WeightClass::Medium) => 13.0,
            (Kind::Stunt { .. }, WeightClass::Heavy) => 11.0,
            (Kind::Flip { .. }, WeightClass::Light) => 20.0,
            (Kind::Flip { .. }, WeightClass::Medium) => 18.0,
            (Kind::Flip { .. }, WeightClass::Heavy) => 16.0,
        }
    }

    fn max_angle(&self) -> f32 {
        match self {
            Kind::Stunt { .. } => 180.0,
            Kind::Flip {
                is_double: false, ..
            } => 360.0,
            Kind::Flip {
                is_double: true, ..
            } => 720.0,
        }
    }

    fn initial_angle_diff(&self) -> f32 {
        match self {
            Kind::Stunt { .. } => 7.5,
            Kind::Flip {
                is_double: false, ..
            } => 11.0,
            Kind::Flip {
                is_double: true, ..
            } => 14.0,
        }
    }

    fn min_angle_diff(&self) -> f32 {
        match self {
            Kind::Stunt { .. } => 2.5,
            Kind::Flip { .. } => 1.5,
        }
    }

    fn min_angle_diff_mul(&self) -> f32 {
        match self {
            Kind::Stunt { .. } => 0.93,
            Kind::Flip { .. } => 0.9,
        }
    }

    fn angle_diff_mul_dec(&self) -> f32 {
        match self {
            Kind::Stunt { .. } => 0.05,
            Kind::Flip {
                is_double: false, ..
            } => 0.0018,
            Kind::Flip {
                is_double: true, ..
            } => 0.0006,
        }
    }

    fn boost_duration(&self, is_bike: bool) -> u16 {
        match &self {
            Kind::Stunt => {
                if is_bike {
                    45
                } else {
                    40
                }
            }
            Kind::Flip {
                is_double: false, ..
            } => {
                if is_bike {
                    80
                } else {
                    70
                }
            }
            Kind::Flip {
                is_double: true, ..
            } => {
                if is_bike {
                    95
                } else {
                    85
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    fn rot(&self, angle: f32) -> Quat {
        let angles = match self {
            Axis::X => Vec3::new(angle, 0.0, 0.0),
            Axis::Y => Vec3::new(0.0, angle, 0.0),
            Axis::Z => Vec3::new(0.0, 0.0, angle),
        };

        Quat::from_angles(angles)
    }
}
