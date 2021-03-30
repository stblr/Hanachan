mod bsp;
mod driver_param;
mod error;
mod kart_param;
mod player;
mod rkg;
mod slice_ext;
mod take;
mod u8;
mod vec3;
mod yaz;

use std::env;

use crate::player::Player;
use crate::rkg::Rkg;
use crate::u8::U8;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: hanachan <Common.szs> <ghost.rkg>");
        return;
    }

    let common_szs = match U8::open_szs(&args[1]) {
        Ok(common_szs) => common_szs,
        Err(_) => {
            eprintln!("Couldn't load Common.szs");
            return;
        }
    };

    let rkg = match Rkg::open(&args[2]) {
        Ok(rkg) => rkg,
        Err(_) => {
            eprintln!("Couldn't load rkg");
            return;
        }
    };

    let player = match Player::try_new(rkg.header().params(), &common_szs) {
        Some(player) => player,
        None => {
            eprintln!("Couldn't initialize player");
            return;
        }
    };
}
