mod bike;
mod boost;
mod drift;
mod handle;
mod lean;
mod params;
mod physics;
mod start_boost;
mod stats;
mod turn;
mod vehicle_body;
mod wheel;
mod wheelie;

pub use handle::Handle;
pub use params::{Character, Params, Vehicle};
pub use stats::{CommonStats, Stats};

use crate::fs::{Rkg, RkgTrick, U8};
use crate::geom::{Mat33, Vec3};
use crate::race::{Race, Stage};
use crate::wii::F32Ext;

use bike::Bike;
use boost::{Boost, Kind as BoostKind};
use drift::Drift;
use lean::Lean;
use physics::Physics;
use start_boost::StartBoost;
use turn::Turn;
use vehicle_body::VehicleBody;
use wheel::Wheel;
use wheelie::Wheelie;

#[derive(Clone, Debug)]
pub struct Player {
    stats: Stats,
    rkg: Rkg,
    airtime: u32,
    start_boost: StartBoost,
    drift: Drift,
    boost: Boost,
    turn: Turn,
    diving_rot: f32,
    standstill_boost_rot: f32, // TODO maybe rename
    bike: Option<Bike>,
    physics: Physics,
    vehicle_body: VehicleBody,
    wheels: Vec<Wheel>,
}

impl Player {
    pub fn try_new(common_szs: &U8, rkg: Rkg) -> Option<Player> {
        let params = rkg.header().params();

        let kart_param = common_szs
            .get_node("./kartParam.bin")?
            .content()
            .as_file()?
            .as_kart_param()?;
        let vehicle_stats = kart_param.vehicle(*params.vehicle());

        let driver_param = common_szs
            .get_node("./driverParam.bin")?
            .content()
            .as_file()?
            .as_driver_param()?;
        let character_stats = driver_param.character(*params.character());

        let stats = vehicle_stats.merge_with(*character_stats);

        let drift = Drift::new(stats.common.mt_duration as u16);

        let turn = Turn::new(&stats.common);

        let bike = stats.vehicle.kind.is_bike().then(|| Bike::new());

        let path = "./bsp/".to_owned() + params.vehicle().filename() + ".bsp";
        let bsp = common_szs.get_node(&path)?.content().as_file()?.as_bsp()?;

        let ktpt_pos = Vec3::new(-14720.0, 1000.0, -2954.655); // TODO load from Kmp
        let physics = Physics::new(stats, bsp, ktpt_pos);

        let vehicle_body = VehicleBody::new(bsp.hitboxes.clone());

        let bike_parts_disp_param = common_szs
            .get_node("./bikePartsDispParam.bin")?
            .content()
            .as_file()?
            .as_bike_parts_disp_param()?;
        let handle = bike_parts_disp_param.vehicle(*params.vehicle());

        let wheel_count = stats.vehicle.kind.wheel_count();
        let wheels = (0..4)
            .filter(|i| wheel_count != 2 || i % 2 == 0)
            .filter(|i| wheel_count != 3 || *i != 0)
            .map(|i| {
                let handle = handle.filter(|_| i == 0);
                let mut bsp_wheel = bsp.wheels[i / 2];
                if i % 2 == 1 {
                    bsp_wheel.mirror_x_pos();
                }
                Wheel::new(handle, bsp_wheel, physics.pos)
            })
            .collect();

        Some(Player {
            stats,
            rkg,
            airtime: 0,
            start_boost: StartBoost::new(),
            drift,
            boost: Boost::new(),
            turn,
            diving_rot: 0.0,
            standstill_boost_rot: 0.0,
            bike,
            physics,
            vehicle_body,
            wheels,
        })
    }

    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn update(&mut self, race: &Race) {
        self.physics.rot_vec2 = Vec3::ZERO;

        let ground = self.wheels.iter().any(|wheel| wheel.floor_nor.is_some());
        if ground {
            self.airtime = 0;
        } else {
            self.airtime += 1;
        }

        if race.stage() == Stage::Countdown {
            self.start_boost.update(self.rkg.accelerate(race.frame()));
        } else if race.frame() == 411 {
            self.boost.activate(BoostKind::Weak, self.start_boost.boost_frames());
        }

        self.physics.update_floor_nor(self.drift.is_hopping(), &self.wheels, ground);

        self.physics.update_dir(self.drift.hop_dir());

        let frame_idx = race.frame();
        let stick_x = self.rkg.stick_x(frame_idx);
        self.turn.update(stick_x, &self.drift);

        let drift = self.rkg.drift(frame_idx);
        let wheelie = self.bike.as_mut().map(|bike| &mut bike.wheelie);
        self.drift.update(drift, stick_x, &mut self.boost, wheelie, &mut self.physics, ground);

        if let Some(bike) = &mut self.bike {
            let base_speed = self.stats.common.base_speed;
            let trick_is_up = self.rkg.trick(frame_idx) == Some(RkgTrick::Up);
            let is_drifting = self.drift.is_drifting();
            bike.wheelie.update(base_speed, trick_is_up, is_drifting, &mut self.physics);
        }

        self.boost.update();

        let is_wheelieing = self
            .bike
            .as_ref()
            .map(|bike| bike.wheelie.is_wheelieing())
            .unwrap_or(false);
        self.physics.update_vel1(
            self.airtime,
            self.drift.is_drifting(),
            &self.boost,
            self.turn.raw(),
            is_wheelieing,
            race,
        );

        self.update_standstill_boost_rot(ground, race);
        if let Some(bike) = &mut self.bike {
            self.physics.rot_vec2.x += self.standstill_boost_rot;

            let stick_x = self.rkg.stick_x(race.frame());
            let drift_stick_x = self.drift.drift_stick_x();
            bike.lean.update(stick_x, drift_stick_x, is_wheelieing, &mut self.physics, race);
        } else {
            self.physics.rot_vec0.x += self.standstill_boost_rot;

            let mat = self.physics.mat();
            let front = Mat33::from(mat) * Vec3::FRONT;
            let front = front.perp_in_plane(self.physics.floor_nor, true);
            let rej = self.physics.vel.rej_unit(front);
            let perp = rej.perp_in_plane(self.physics.floor_nor, false);
            let sq_norm = perp.sq_norm();
            let norm = if sq_norm > f32::EPSILON && ground {
                -sq_norm.wii_sqrt().min(1.0) * (perp.x * front.z - perp.z * front.x).signum()
            } else {
                0.0
            };
            self.physics.rot_vec0.z += self.stats.common.tilt_factor * norm * self.turn.raw().abs();
        }

        self.turn.update_rot(&self.drift, is_wheelieing, &mut self.physics);

        self.diving_rot *= 0.96;
        if !ground {
            let stick_y = self.rkg.stick_y(race.frame());
            let diving_rot_diff = if self.airtime > 50 {
                stick_y
            } else {
                self.airtime as f32 / 50.0 * stick_y
            };
            self.diving_rot += 0.005 * diving_rot_diff;
            self.physics.rot_vec2.x += self.diving_rot;
        }

        self.physics.update(race);

        self.vehicle_body.update(&mut self.physics);

        for wheel in &mut self.wheels {
            wheel.update(self.bike.as_ref(), &mut self.physics);
        }

        self.drift.update_hop_physics();

        self.physics.update_mat();

        let last_use_item = race
            .frame()
            .checked_sub(1)
            .map(|last_frame| self.rkg.use_item(last_frame))
            .unwrap_or(false);
        if self.rkg.use_item(race.frame()) && !last_use_item {
            self.boost.activate(BoostKind::Strong, 90);
        }
    }

    fn update_standstill_boost_rot(&mut self, ground: bool, race: &Race) {
        if !ground {
            self.standstill_boost_rot = 0.0;
        } else if race.stage() == Stage::Countdown {
            let inc = 0.015 * -self.start_boost.charge - self.standstill_boost_rot;
            self.standstill_boost_rot += inc;
        } else {
            let acceleration = (self.physics.speed1 - self.physics.last_speed1).clamp(-3.0, 3.0);

            let is_bike = self.stats.vehicle.kind.is_bike();
            let vehicle_factor = if is_bike { 0.2 } else { 1.0 };

            let is_wheelieing = self
                .bike
                .as_ref()
                .map(|bike| bike.wheelie.is_wheelieing())
                .unwrap_or(false);
            let (boost_factor, wheelie_factor) = if self.boost.is_strong() {
                let wheelie_factor = if is_wheelieing { 0.5 } else { 1.0 };
                (0.25, wheelie_factor)
            } else {
                (0.08, 1.0)
            };

            let val = -acceleration * 0.15 * boost_factor * wheelie_factor;
            let inc = vehicle_factor * (val - self.standstill_boost_rot);
            self.standstill_boost_rot += inc;
        }
    }
}
