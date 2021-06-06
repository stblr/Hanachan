mod error;
mod fs;
mod geom;
mod player;
mod race;
mod track;
mod tracks;
mod wii;

use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;

use crate::error::Error;
use crate::fs::{yaz, Rkg, Rkrd, SliceRefExt, U8};
use crate::player::Player;
use crate::race::Race;
use crate::tracks::Tracks;

fn main() {
    enable_flushing_denormals_to_zero();

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: hanachan <Common.szs> <track(s)> <ghost(s)>");
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

    let mut tracks = match Tracks::try_new(&args[2]) {
        Ok(tracks) => tracks,
        Err(_) => {
            eprintln!("Couldn't load track file or directory");
            return;
        }
    };

    let metadata = match std::fs::metadata(&args[3]) {
        Ok(metadata) => metadata,
        Err(_) => {
            eprintln!("Couldn't open rkg file or directory");
            return;
        }
    };
    if metadata.is_dir() {
        let dir = match std::fs::read_dir(&args[3]) {
            Ok(dir) => dir,
            Err(_) => {
                eprintln!("Couldn't open rkg directory");
                return;
            }
        };
        let mut rkg_paths: Vec<_> = dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension() == Some(OsStr::new("rkg")))
            .collect();
        rkg_paths.sort();
        for rkg_path in rkg_paths {
            replay_rkg(&common_szs, &mut tracks, &rkg_path, false);
        }
    } else {
        replay_rkg(&common_szs, &mut tracks, &PathBuf::from(&args[3]), true);
    }
}

fn enable_flushing_denormals_to_zero() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        extern "C" {
            fn enable_ftz();
        }

        enable_ftz();
    }
}

fn replay_rkg(common_szs: &U8, tracks: &mut Tracks, rkg_path: &PathBuf, verbose: bool) {
    let mut rkg: &[u8] = &match std::fs::read(rkg_path) {
        Ok(rkg) => rkg,
        Err(_) => {
            eprintln!("Couldn't open rkg");
            return;
        }
    };
    let rkg: Rkg = match rkg.take() {
        Ok(rkg) => rkg,
        Err(_) => {
            eprintln!("Couldn't parse rkg");
            return;
        }
    };

    let track = match tracks.get(rkg.header().track_id) {
        Ok(track) => track,
        Err(_) => {
            eprintln!("Couldn't load track");
            return;
        }
    };

    let player = match Player::try_new(&common_szs, &track, rkg) {
        Some(player) => player,
        None => {
            eprintln!("Couldn't initialize player");
            return;
        }
    };

    let rkrd_path = rkg_path.with_extension("rkrd");
    let mut rkrd: &[u8] = &match std::fs::read(rkrd_path) {
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

    let mut race = Race::new(&track, player);
    let mut desync = false;
    for frame in rkrd.frames() {
        race.update();

        let physics = race.player().physics();
        check_val("up", physics.up, frame.floor_nor, &mut desync, verbose);
        check_val("dir", physics.dir, frame.dir, &mut desync, verbose);
        check_val("pos", physics.pos, frame.pos, &mut desync, verbose);
        check_val("vel0", physics.vel0, frame.vel0, &mut desync, verbose);
        check_val("speed1", physics.speed1, frame.speed1, &mut desync, verbose);
        check_val("vel", physics.vel, frame.vel, &mut desync, verbose);
        check_val("rot_vec0", physics.rot_vec0, frame.rot_vec0, &mut desync, verbose);
        check_val("rot_vec2", physics.rot_vec2, frame.rot_vec2, &mut desync, verbose);
        check_val("rot0", physics.rot0, frame.rot0, &mut desync, verbose);
        check_val("rot1", physics.rot1, frame.rot1, &mut desync, verbose);

        if desync {
            break;
        }
    }

    if let Some(run_name) = rkg_path.file_stem().and_then(|run_name| run_name.to_str()) {
        println!("{}: {} / {}", run_name, race.frame_idx() - 1, rkrd.frames().len());
    }
}

fn check_val<T: Debug + PartialEq>(
    name: &str,
    actual: T,
    expected: T,
    desync: &mut bool,
    verbose: bool,
) {
    if actual != expected {
        if verbose {
            println!("{}", name);
            println!("{:?}", actual);
            println!("{:?}", expected);
        }

        *desync = true;
    }
}
