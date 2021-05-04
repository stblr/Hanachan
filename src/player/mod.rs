mod handle;
mod params;
mod physics;
mod start_boost;
mod stats;
mod wheel;

pub use handle::Handle;
pub use params::{Character, Params, Vehicle};
pub use stats::{CommonStats, Stats};

use crate::fs::{Rkg, U8};
use crate::geom::{Mat33, Vec3};
use crate::race::{Race, Stage};
use crate::wii::F32Ext;

use physics::Physics;
use start_boost::StartBoost;
use wheel::Wheel;

#[derive(Clone, Debug)]
pub struct Player {
    stats: Stats,
    rkg: Rkg,
    start_boost: StartBoost,
    boost_frames: u16,
    turn: f32,
    standstill_boost_rot: f32, // TODO maybe rename
    physics: Physics,
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

        let path = "./bsp/".to_owned() + params.vehicle().filename() + ".bsp";
        let bsp = common_szs.get_node(&path)?.content().as_file()?.as_bsp()?;

        let mut pos = Vec3::new(-14720.0, 1000.0, -2954.655); // TODO load from Kmp
        pos.y += bsp.initial_pos_y;
        let physics = Physics::new(stats, bsp.cuboids, bsp.rot_factor, pos);

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
                Wheel::new(handle, bsp_wheel, pos)
            })
            .collect();

        Some(Player {
            stats,
            rkg,
            start_boost: StartBoost::new(),
            boost_frames: 0,
            turn: 0.0,
            standstill_boost_rot: 0.0,
            physics,
            wheels,
        })
    }

    pub fn physics(&self) -> Physics {
        self.physics
    }

    pub fn update(&mut self, race: &Race) {
        if race.stage() == Stage::Countdown {
            self.start_boost.update(self.rkg.accelerate(race.frame()));
        } else if race.frame() == 411 {
            self.boost_frames = self.start_boost.boost_frames();
        }

        self.update_turn(race);

        let is_boosting = self.boost_frames > 0;
        if is_boosting {
            self.boost_frames -= 1;
        }

        self.physics.update_speed1(is_boosting, race);

        self.physics.rot_vec2 = Vec3::ZERO;

        self.update_standstill_boost_rot(race);
        let is_bike = self.stats.vehicle.kind.is_bike();
        if is_bike {
            self.physics.rot_vec2.x += self.standstill_boost_rot;
        } else {
            self.physics.rot_vec0.x += self.standstill_boost_rot;

            let mat = self.physics.mat();
            let front = Mat33::from(mat) * Vec3::FRONT;
            let front = front.perp_in_plane(self.physics.floor_nor, true);
            let rej = self.physics.vel.rej_unit(front);
            let perp = rej.perp_in_plane(self.physics.floor_nor, false);
            let sq_norm = perp.sq_norm();
            let norm = if sq_norm > f32::EPSILON {
                -sq_norm.wii_sqrt().min(1.0) * (perp.x * front.z - perp.z * front.x).signum()
            } else {
                0.0
            };
            self.physics.rot_vec0.z += self.stats.common.tilt_factor * norm * self.turn.abs();
        }

        let tightness = self.stats.common.manual_handling_tightness; // TODO handle drift
        let turn = self.turn * tightness;
        let turn = if self.physics.speed1 < 20.0 {
            0.4 * turn + (self.physics.speed1 / 20.0) * (turn * 0.6)
        } else if self.physics.speed1 < 70.0 {
            0.5 * turn + (1.0 - (self.physics.speed1 - 20.0) / (70.0 - 20.0)) * (turn * 0.5)
        } else {
            0.5 * turn
        };
        self.physics.rot_vec2.y += turn;

        self.physics.update(is_bike, &self.wheels, race);

        for wheel in &mut self.wheels {
            wheel.update(&mut self.physics);
        }
    }

    fn update_turn(&mut self, race: &Race) {
        let stick_x = self.rkg.stick_x(race.frame());
        let reactivity = self.stats.common.handling_reactivity; // TODO handle drift
        self.turn = reactivity * -stick_x + (1.0 - reactivity) * self.turn;
    }

    fn update_standstill_boost_rot(&mut self, race: &Race) {
        if race.stage() == Stage::Countdown {
            let inc = 0.015 * -self.start_boost.charge - self.standstill_boost_rot;
            self.standstill_boost_rot += inc;
        } else {
            let acceleration = (self.physics.speed1 - self.physics.last_speed1).clamp(-3.0, 3.0);
            let factor = 1.0; // TODO handle bikes
            let inc = factor * (-acceleration * 0.15 * 0.08 - self.standstill_boost_rot);
            self.standstill_boost_rot += inc;
        }
    }
}
