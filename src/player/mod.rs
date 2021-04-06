mod handle;
mod params;
mod physics;
mod stats;
mod wheel;

pub use handle::Handle;
pub use params::{Character, Params, Vehicle};
pub use stats::{CommonStats, Stats};

use crate::fs::{Rkg, U8};
use crate::geom::Vec3;
use crate::race::{Race, Stage};

use physics::Physics;
use wheel::Wheel;

#[derive(Clone, Debug)]
pub struct Player {
    rkg: Rkg,
    start_boost_charge: f32,
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
        let physics = Physics::new(bsp.cuboids, bsp.rot_factor, pos);

        let bike_parts_disp_param = common_szs
            .get_node("./bikePartsDispParam.bin")?
            .content()
            .as_file()?
            .as_bike_parts_disp_param()?;
        let handle = bike_parts_disp_param.vehicle(*params.vehicle());

        let wheel_count = stats.wheel_count();
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
            rkg,
            start_boost_charge: 0.0,
            physics,
            wheels,
        })
    }

    pub fn physics(&self) -> Physics {
        self.physics
    }

    pub fn update(&mut self, race: &Race) {
        if race.stage() == Stage::Countdown {
            self.update_start_boost_charge(race);
        }

        self.physics.update(&self.wheels);

        for wheel in &mut self.wheels {
            wheel.update(&mut self.physics);
        }
    }

    fn update_start_boost_charge(&mut self, race: &Race) {
        if self.rkg.accelerate(race.frame()) {
            self.start_boost_charge += 0.02 - (0.02 - 0.002) * self.start_boost_charge;
        } else {
            self.start_boost_charge *= 0.96;
        }
        self.start_boost_charge = self.start_boost_charge.clamp(0.0, 1.0);
    }
}
