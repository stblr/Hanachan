mod params;
mod stats;

pub use params::{Character, Params, Vehicle};
pub use stats::{CommonStats, Stats};

use crate::u8::U8;

#[derive(Clone, Debug)]
pub struct Player {}

impl Player {
    pub fn try_new(params: &Params, common_szs: &U8) -> Option<Player> {
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
        println!("{:#?}", stats);

        let path = "./bsp/".to_owned() + params.vehicle().filename() + ".bsp";
        let bsp = common_szs.get_node(&path)?.content().as_file()?.as_bsp()?;
        println!("{:#?}", bsp);

        Some(Player {})
    }
}
