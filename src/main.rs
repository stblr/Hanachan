mod bsp;
mod error;
mod rkg;
mod slice_ext;
mod take;
mod u8;
mod vec3;
mod yaz;

use std::env;

use crate::rkg::Rkg;
use crate::u8::U8;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: hanachan <Common.szs> <ghost.rkg>");
        return;
    }

    let common_szs = U8::open_szs(&args[1]).unwrap();
    let bsp = common_szs.get_node("./bsp/se_bike.bsp");
    println!("{:#?}", bsp);

    match Rkg::open(&args[2]) {
        Ok(rkg) => eprintln!("{:#?}", rkg.header()),
        Err(error) => eprintln!("{:#?}", error),
    }
}
