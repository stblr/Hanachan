mod error;
mod rkg;
mod view;
mod yaz;

use std::env;

use rkg::Rkg;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: hanachan <ghost.rkg>");
        return;
    }

    match Rkg::open(&args[1]) {
        Ok(rkg) => eprintln!("{:#?}", rkg.header()),
        Err(error) => eprintln!("{:#?}", error),
    }
}
