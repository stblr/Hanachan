mod bike;
mod boost;
mod boost_ramp;
mod collision;
mod drift;
mod floor_factors;
mod handle;
mod jump_pad;
mod lean;
mod params;
mod physics;
mod start_boost;
mod stats;
mod sticky_road;
mod turn;
mod vehicle_body;
mod wheel;
mod wheelie;

pub use handle::Handle;
pub use params::{Character, Params, Vehicle};
pub use stats::{CommonStats, Stats};

use std::iter;

use crate::fs::{Kcl, Rkg, U8};
use crate::geom::{Mat33, Vec3};
use crate::race::{Stage, Timer};
use crate::track::Track;
use crate::wii::F32Ext;

use bike::Bike;
use boost::{Boost, Kind as BoostKind};
use boost_ramp::BoostRamp;
use collision::Collision;
use drift::Drift;
use floor_factors::FloorFactors;
use jump_pad::JumpPad;
use lean::Lean;
use physics::Physics;
use start_boost::StartBoost;
use sticky_road::StickyRoad;
use turn::Turn;
use vehicle_body::VehicleBody;
use wheel::Wheel;
use wheelie::Wheelie;

#[derive(Clone, Debug)]
pub struct Player {
    stats: Stats,
    rkg: Rkg,
    airtime: u32,
    floor_factors: FloorFactors,
    start_boost: StartBoost,
    drift: Drift,
    boost: Boost,
    turn: Turn,
    diving_rot: f32,
    mushroom_boost: u16,
    standstill_boost_rot: f32, // TODO maybe rename
    boost_ramp: BoostRamp,
    jump_pad: JumpPad,
    bike: Option<Bike>,
    sticky_road: StickyRoad,
    physics: Physics,
    vehicle_body: VehicleBody,
    wheels: Vec<Wheel>,
}

impl Player {
    pub fn try_new(common_szs: &U8, track: &Track, rkg: Rkg) -> Option<Player> {
        let params = rkg.header().params;

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

        let physics = Physics::new(bsp, track);

        let vehicle_body = VehicleBody::new(bsp.hitboxes.clone(), &physics);

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
            floor_factors: FloorFactors::new(),
            start_boost: StartBoost::new(),
            drift,
            boost: Boost::new(),
            turn,
            diving_rot: 0.0,
            mushroom_boost: 0,
            standstill_boost_rot: 0.0,
            boost_ramp: BoostRamp::new(),
            jump_pad: JumpPad::new(),
            bike,
            sticky_road: StickyRoad::new(),
            physics,
            vehicle_body,
            wheels,
        })
    }

    pub fn physics(&self) -> &Physics {
        &self.physics
    }

    pub fn update(&mut self, kcl: &Kcl, timer: &Timer) {
        self.physics.rot_vec2 = Vec3::ZERO;

        let collisions = self
            .wheels
            .iter()
            .map(|wheel| wheel.collision())
            .chain(iter::once(self.vehicle_body.collision()));

        let ground = collisions.clone().any(|collision| collision.floor_nor().is_some());
        let last_airtime = self.airtime;
        if ground {
            self.airtime = 0;
        } else {
            self.airtime += 1;
        }

        if timer.stage() == Stage::Countdown {
            self.start_boost.update(self.rkg.accelerate(timer.frame_idx()));
        } else if timer.frame_idx() == 411 {
            self.boost.activate(BoostKind::Weak, self.start_boost.boost_frames());
        }

        let is_wheelieing = self
            .bike
            .as_ref()
            .map(|bike| bike.wheelie.is_wheelieing())
            .unwrap_or(false);
        self.physics.update_floor_nor(
            self.stats.vehicle.drift_kind.is_inside(),
            self.airtime,
            last_airtime,
            self.drift.has_hop_height(),
            is_wheelieing,
            self.boost.is_boosting(),
            collisions.clone(),
        );

        if self.airtime == 0 && last_airtime != 0 {
            self.jump_pad.end();
        }

        let has_boost_panel = collisions.clone().any(Collision::has_boost_panel);
        if has_boost_panel {
            self.boost.activate(BoostKind::Strong, 60);
            self.floor_factors.activate_invicibility(60);
        }

        self.boost_ramp.try_start(collisions.clone());

        self.jump_pad.try_start(&mut self.physics, collisions.clone());

        self.physics.update_dir(
            self.airtime,
            last_airtime,
            self.floor_factors.rot_factor(),
            &self.drift,
            self.jump_pad.enabled()
        );

        self.sticky_road.update(&mut self.physics, collisions.clone(), kcl);

        self.floor_factors.update_factors(
            &self.stats.common,
            &self.vehicle_body,
            &self.wheels,
            collisions.clone(),
        );

        let frame_idx = timer.frame_idx();
        let stick_x = self.rkg.stick_x(frame_idx);
        self.turn.update(&self.stats.common, self.airtime, stick_x, &self.drift);

        let drift_input = self.rkg.drift(frame_idx) && timer.stage() == Stage::Race;
        let last_drift_input = timer
            .frame_idx()
            .checked_sub(1)
            .map(|last_frame_idx| self.rkg.drift(last_frame_idx))
            .unwrap_or(false);
        let wheelie = self.bike.as_mut().map(|bike| &mut bike.wheelie);
        self.drift.update(
            &self.stats,
            drift_input,
            last_drift_input,
            stick_x,
            self.airtime,
            &mut self.boost,
            wheelie,
            &mut self.physics,
        );

        if let Some(bike) = &mut self.bike {
            bike.wheelie.update(
                self.stats.common.base_speed,
                self.rkg.trick(frame_idx),
                ground,
                &self.drift,
                &mut self.physics,
            );
        }

        self.boost.update();

        self.boost_ramp.update();

        self.mushroom_boost = self.mushroom_boost.saturating_sub(1);

        self.floor_factors.update_invicibility();

        let last_accelerate = timer
            .frame_idx()
            .checked_sub(1)
            .map(|last_frame_idx| self.rkg.accelerate(last_frame_idx))
            .unwrap_or(false);
        let last_brake = timer
            .frame_idx()
            .checked_sub(1)
            .map(|last_frame_idx| self.rkg.brake(last_frame_idx))
            .unwrap_or(false);
        let is_wheelieing = self
            .bike
            .as_ref()
            .map(|bike| bike.wheelie.is_wheelieing())
            .unwrap_or(false);
        self.physics.update_vel1(
            &self.stats,
            self.rkg.accelerate(frame_idx),
            self.rkg.brake(frame_idx),
            last_accelerate,
            last_brake,
            self.airtime,
            self.floor_factors.speed_factor(),
            self.drift.is_drifting(),
            &self.boost,
            self.turn.raw(),
            self.boost_ramp.enabled(),
            self.jump_pad.speed(),
            is_wheelieing,
            collisions,
            timer,
        );

        self.update_standstill_boost_rot(
            ground,
            self.boost_ramp.enabled(),
            self.jump_pad.enabled(),
            timer,
        );

        if let Some(bike) = &mut self.bike {
            self.physics.rot_vec2.x += self.standstill_boost_rot;

            bike.lean.update(
                self.rkg.stick_x(timer.frame_idx()),
                self.airtime,
                self.drift.drift_stick_x(),
                is_wheelieing,
                &mut self.physics,
                timer,
            );
        } else {
            self.physics.rot_vec0.x += self.standstill_boost_rot;

            let mut norm = 0.0;
            if ground {
                let mat = self.physics.mat();
                let front = Mat33::from(mat) * Vec3::FRONT;
                let front = front.perp_in_plane(self.physics.up, true);
                let rej = self.physics.vel.rej_unit(front);
                let perp = rej.perp_in_plane(self.physics.up, false);
                let sq_norm = perp.sq_norm();
                if sq_norm > f32::EPSILON {
                    let det = perp.x * front.z - perp.z * front.x;
                    norm = -sq_norm.wii_sqrt().min(1.0) * det.signum();
                }
            } else if !self.drift.has_hop_height() {
                self.physics.rot_vec0.z *= 0.98;
            }
            self.physics.rot_vec0.z += self.stats.common.tilt_factor * norm * self.turn.raw().abs();
        }

        self.turn.update_rot(
            &self.stats.common,
            self.airtime,
            &self.drift,
            is_wheelieing,
            &mut self.physics,
        );

        self.diving_rot *= 0.96;
        if !ground {
            let stick_y = self.rkg.stick_y(timer.frame_idx());
            let diving_rot_diff = if timer.stage() != Stage::Race {
                0.0
            } else if self.airtime > 50 {
                stick_y
            } else {
                self.airtime as f32 / 50.0 * stick_y
            };
            self.diving_rot += 0.005 * diving_rot_diff;
            self.physics.rot_vec2.x += self.diving_rot;
        }

        self.physics.update(&self.stats, timer);

        self.vehicle_body.update(
            &self.stats.common,
            self.boost.is_boosting(),
            &mut self.physics,
            &kcl,
        );

        let mut count = 0;
        let (mut min, mut max) = (Vec3::ZERO, Vec3::ZERO);
        let mut pos_rel = Vec3::ZERO;
        let mut vel = Vec3::ZERO;
        let mut floor_nor = Vec3::ZERO;
        for wheel in &mut self.wheels {
            let vehicle_movement = wheel.update(
                &self.stats.common,
                self.bike.as_ref(),
                &mut self.physics,
                &kcl,
            );

            if let Some(vehicle_movement) = vehicle_movement {
                min = min.min(vehicle_movement);
                max = max.max(vehicle_movement);

                if let Some(wheel_floor_nor) = wheel.collision().floor_nor() {
                    count += 1;
                    pos_rel += wheel.hitbox_pos_rel();
                    vel += 10.0 * 1.3 * Vec3::DOWN;
                    floor_nor += wheel_floor_nor;
                }
            }
        }

        let vehicle_movement = min + max;
        self.physics.pos += vehicle_movement;

        if count > 0 && self.vehicle_body.collision().floor_nor().is_none() {
            let pos_rel = (1.0 / count as f32) * pos_rel;
            let vel = (1.0 / count as f32) * vel;
            let floor_nor = floor_nor.normalize();
            self.physics.apply_rigid_body_motion(self.boost.is_boosting(), pos_rel, vel, floor_nor);
            self.vehicle_body.insert_floor_nor(floor_nor);
        }

        for wheel in &mut self.wheels {
            wheel.apply_suspension(self.bike.as_ref(), &mut self.physics, vehicle_movement);
        }

        self.drift.update_hop_physics();

        self.physics.update_mat();

        let last_use_item = timer
            .frame_idx()
            .checked_sub(1)
            .map(|last_frame_idx| self.rkg.use_item(last_frame_idx))
            .unwrap_or(false);
        if self.rkg.use_item(timer.frame_idx()) && !last_use_item {
            self.boost.activate(BoostKind::Strong, 90);
            self.floor_factors.activate_invicibility(90);
            self.mushroom_boost = 90;
        }
    }

    fn update_standstill_boost_rot(
        &mut self,
        ground: bool,
        boost_ramp_enabled: bool,
        jump_pad_enabled: bool,
        timer: &Timer,
    ) {
        let mut next = 0.0;
        let mut t = 1.0;
        if ground {
            if timer.stage() == Stage::Countdown {
                next = 0.015 * -self.start_boost.charge;
            } else if !boost_ramp_enabled && !jump_pad_enabled {
                let acceleration = self.physics.speed1 - self.physics.last_speed1;
                let acceleration = acceleration.clamp(-3.0, 3.0);

                if self.mushroom_boost > 0 {
                    next = -acceleration * 0.15 * 0.25;
                    let is_wheelieing = self
                        .bike
                        .as_ref()
                        .map(|bike| bike.wheelie.is_wheelieing())
                        .unwrap_or(false);
                    if is_wheelieing {
                        next *= 0.5;
                    }
                } else {
                    next = -acceleration * 0.15 * 0.08;
                }

                let is_bike = self.stats.vehicle.drift_kind.is_bike();
                t = if is_bike { 0.2 } else { 1.0 };
            }
        }

        self.standstill_boost_rot += t * (next - self.standstill_boost_rot);
    }
}
