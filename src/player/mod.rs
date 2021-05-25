mod bike;
mod boost;
mod collision;
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

use std::ops::Add;

use crate::fs::{kmp::Ktpt, Kcl, Rkg, RkgTrick, U8};
use crate::geom::{Mat33, Vec3};
use crate::race::{Race, Stage};
use crate::wii::F32Ext;

use bike::Bike;
use boost::{Boost, Kind as BoostKind};
use collision::Collision;
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
    kcl_speed_factor: f32,
    kcl_rot_factor: f32,
    start_boost: StartBoost,
    drift: Drift,
    boost: Boost,
    offroad_invicibility: u16,
    turn: Turn,
    diving_rot: f32,
    mushroom_boost: u16,
    standstill_boost_rot: f32, // TODO maybe rename
    bike: Option<Bike>,
    physics: Physics,
    vehicle_body: VehicleBody,
    wheels: Vec<Wheel>,
}

impl Player {
    pub fn try_new(common_szs: &U8, ktpt: Ktpt, kcl: &Kcl, rkg: Rkg) -> Option<Player> {
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

        let drift = Drift::new(&stats);

        let turn = Turn::new();

        let drift_kind = stats.vehicle.drift_kind;
        let bike = drift_kind.is_bike().then(|| Bike::new(drift_kind.is_inside()));

        let path = "./bsp/".to_owned() + params.vehicle().filename() + ".bsp";
        let bsp = common_szs.get_node(&path)?.content().as_file()?.as_bsp()?;

        let physics = Physics::new(bsp, ktpt.pos, kcl);

        let vehicle_body = VehicleBody::new(bsp.hitboxes.clone());

        let bike_parts_disp_param = common_szs
            .get_node("./bikePartsDispParam.bin")?
            .content()
            .as_file()?
            .as_bike_parts_disp_param()?;
        let has_handle = stats.vehicle.has_handle;
        let handle = bike_parts_disp_param.vehicle(*params.vehicle()).filter(|_| has_handle);

        let wheel_count = stats.vehicle.wheel_count;
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
            kcl_speed_factor: 1.0,
            kcl_rot_factor: 1.0,
            start_boost: StartBoost::new(),
            drift,
            boost: Boost::new(),
            offroad_invicibility: 0,
            turn,
            diving_rot: 0.0,
            mushroom_boost: 0,
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

    pub fn update(&mut self, race: &Race, kcl: &Kcl) {
        self.physics.rot_vec2 = Vec3::ZERO;

        let wheel_collision_count = self.wheels.iter().filter(|wheel| wheel.collision().is_some()).count();
        let ground = wheel_collision_count > 0;
        let (is_landing, airtime) = if ground {
            (self.airtime > 3, 0)
        } else {
            (false, self.airtime + 1)
        };
        self.airtime = airtime;

        if race.stage() == Stage::Countdown {
            self.start_boost.update(self.rkg.accelerate(race.frame()));
        } else if race.frame() == 411 {
            self.boost.activate(BoostKind::Weak, self.start_boost.boost_frames());
        }

        let is_wheelieing = self
            .bike
            .as_ref()
            .map(|bike| bike.wheelie.is_wheelieing())
            .unwrap_or(false);
        self.physics.update_floor_nor(
            self.stats.vehicle.drift_kind.is_inside(),
            airtime,
            is_landing,
            self.drift.is_hopping(),
            is_wheelieing,
            self.boost.is_boosting(),
            &self.vehicle_body,
            &self.wheels,
        );

        let has_boost_panel = self
            .wheels
            .iter()
            .filter_map(|wheel| wheel.collision())
            .any(|collision| collision.has_boost_panel);
        if has_boost_panel {
            self.boost.activate(BoostKind::Strong, 60);
            self.offroad_invicibility = self.offroad_invicibility.max(60);
        }

        self.physics.update_dir(self.airtime, self.kcl_rot_factor, &self.drift);

        let kcl_speed_factor_sum = self.wheels
            .iter()
            .filter_map(|wheel| wheel.collision())
            .map(|collision| collision.speed_factor)
            .reduce(Add::add);
        if self.offroad_invicibility > 0 {
            self.kcl_speed_factor = self.stats.common.kcl_speed_factors[0];
        } else if let Some(kcl_speed_factor_sum) = kcl_speed_factor_sum {
            self.kcl_speed_factor = kcl_speed_factor_sum / wheel_collision_count as f32;
        }

        let kcl_rot_factor_sum = self.wheels
            .iter()
            .filter_map(|wheel| wheel.collision())
            .map(|collision| collision.rot_factor)
            .reduce(Add::add);
        if self.offroad_invicibility > 0 {
            self.kcl_rot_factor = self.stats.common.kcl_rot_factors[0];
        } else if let Some(kcl_rot_factor_sum) = kcl_rot_factor_sum {
            self.kcl_rot_factor = kcl_rot_factor_sum / wheel_collision_count as f32;
        }

        let frame_idx = race.frame();
        let stick_x = self.rkg.stick_x(frame_idx);
        self.turn.update(&self.stats.common, stick_x, &self.drift);

        let drift = self.rkg.drift(frame_idx);
        let wheelie = self.bike.as_mut().map(|bike| &mut bike.wheelie);
        self.drift.update(
            &self.stats,
            drift,
            stick_x,
            self.airtime,
            &mut self.boost,
            wheelie,
            &mut self.physics,
        );

        if let Some(bike) = &mut self.bike {
            let base_speed = self.stats.common.base_speed;
            let trick_is_up = self.rkg.trick(frame_idx) == Some(RkgTrick::Up);
            let is_drifting = self.drift.is_drifting();
            bike.wheelie.update(base_speed, trick_is_up, is_drifting, &mut self.physics);
        }

        self.boost.update();

        self.mushroom_boost = self.mushroom_boost.saturating_sub(1);

        self.offroad_invicibility = self.offroad_invicibility.saturating_sub(1);

        let is_wheelieing = self
            .bike
            .as_ref()
            .map(|bike| bike.wheelie.is_wheelieing())
            .unwrap_or(false);
        self.physics.update_vel1(
            &self.stats,
            self.airtime,
            self.kcl_speed_factor,
            self.drift.is_drifting(),
            &self.boost,
            self.turn.raw(),
            is_wheelieing,
            race,
        );

        self.update_standstill_boost_rot(ground, race);
        if let Some(bike) = &mut self.bike {
            self.physics.rot_vec2.x += self.standstill_boost_rot;

            bike.lean.update(
                self.rkg.stick_x(race.frame()),
                airtime,
                self.drift.drift_stick_x(),
                is_wheelieing,
                &mut self.physics,
                race,
            );
        } else {
            self.physics.rot_vec0.x += self.standstill_boost_rot;

            let mat = self.physics.mat();
            let front = Mat33::from(mat) * Vec3::FRONT;
            let front = front.perp_in_plane(self.physics.up, true);
            let rej = self.physics.vel.rej_unit(front);
            let perp = rej.perp_in_plane(self.physics.up, false);
            let sq_norm = perp.sq_norm();
            let norm = if sq_norm > f32::EPSILON && ground {
                -sq_norm.wii_sqrt().min(1.0) * (perp.x * front.z - perp.z * front.x).signum()
            } else {
                0.0
            };
            self.physics.rot_vec0.z += self.stats.common.tilt_factor * norm * self.turn.raw().abs();
        }

        self.turn.update_rot(&self.stats.common, &self.drift, is_wheelieing, &mut self.physics);

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

        self.physics.update(&self.stats, race);

        self.vehicle_body.update(&mut self.physics, &kcl);

        for wheel in &mut self.wheels {
            wheel.update(&self.stats.common, self.bike.as_ref(), &mut self.physics, &kcl);
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
            self.offroad_invicibility = self.offroad_invicibility.max(90);
            self.mushroom_boost = 90;
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

            let is_bike = self.stats.vehicle.drift_kind.is_bike();
            let vehicle_factor = if is_bike { 0.2 } else { 1.0 };

            let is_wheelieing = self
                .bike
                .as_ref()
                .map(|bike| bike.wheelie.is_wheelieing())
                .unwrap_or(false);
            let (boost_factor, wheelie_factor) = if self.mushroom_boost > 0 {
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
