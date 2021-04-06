mod fs;
mod geom;
mod player;
mod race;
mod wii;

use std::arch::x86_64;
use std::env;
use std::fmt::Debug;

use crate::fs::{yaz, Rkrd, SliceRefExt};
use crate::player::Player;
use crate::race::Race;

fn main() {
    #[cfg(target_feature = "sse")]
    unsafe {
        x86_64::_MM_SET_FLUSH_ZERO_MODE(x86_64::_MM_FLUSH_ZERO_ON);
    }

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: hanachan <Common.szs> <ghost.rkg> <dump.rkrd>");
        return;
    }

    let common_szs = match std::fs::read(&args[1]) {
        Ok(common_szs) => common_szs,
        Err(_) => {
            eprintln!("Couldn't open Common.szs");
            return;
        }
    };
    let mut common_szs: &[u8] = &match yaz::decompress(&common_szs) {
        Ok(common_szs) => common_szs,
        Err(_) => {
            eprintln!("Couldn't decompress Common.szs");
            return;
        }
    };
    let common_szs = match common_szs.take() {
        Ok(common_szs) => common_szs,
        Err(_) => {
            eprintln!("Couldn't parse Common.szs");
            return;
        }
    };

    let mut rkg: &[u8] = &match std::fs::read(&args[2]) {
        Ok(rkg) => rkg,
        Err(_) => {
            eprintln!("Couldn't open rkg");
            return;
        }
    };
    let rkg = match rkg.take() {
        Ok(rkg) => rkg,
        Err(_) => {
            eprintln!("Couldn't parse rkg");
            return;
        }
    };

    let mut player = match Player::try_new(&common_szs, rkg) {
        Some(player) => player,
        None => {
            eprintln!("Couldn't initialize player");
            return;
        }
    };

    let mut rkrd: &[u8] = &match std::fs::read(&args[3]) {
        Ok(rkrd) => rkrd,
        Err(_) => {
            eprintln!("Couldn't open rkrd");
            return;
        }
    };
    let rkrd = match rkrd.take::<Rkrd>() {
        Ok(rkrd) => rkrd,
        Err(_) => {
            eprintln!("Couldn't parse rkrd");
            return;
        }
    };

    let mut race = Race::new();
    let mut desync = false;
    for frame in rkrd.frames() {
        player.update(&race);
        let physics = player.physics();
        desync = check_val("DIR", race.frame(), physics.dir, frame.dir) || desync;
        desync = check_val("POS", race.frame(), physics.pos, frame.pos) || desync;
        desync = check_val("VEL0", race.frame(), physics.vel0, frame.vel0) || desync;
        desync = check_val("VEL", race.frame(), physics.vel, frame.vel) || desync;
        desync = check_val("ROT_VEC0", race.frame(), physics.rot_vec0, frame.rot_vec0) || desync;
        desync = check_val("ROT0", race.frame(), physics.rot0, frame.rot0) || desync;
        desync = check_val("ROT1", race.frame(), physics.rot1, frame.rot1) || desync;
        if desync {
            break;
        }
        race.update();
    }
}

fn check_val<T: Debug + PartialEq>(name: &str, frame: u32, actual: T, expected: T) -> bool {
    let desync = actual != expected;
    if desync {
        println!("{} {}", name, frame);
        println!("{:?}", actual);
        println!("{:?}", expected);
    }
    desync
}
