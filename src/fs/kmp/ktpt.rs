use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Ktpt {
    pub pos: Vec3,
    pub angles: Vec3,
}

impl Parse for Ktpt {
    fn parse(input: &mut &[u8]) -> Result<Ktpt, Error> {
        let pos = input.take()?;
        let angles = input.take::<Vec3>()?.to_radians();

        input.skip(0x1c - 0xc * 2)?;

        Ok(Ktpt { pos, angles })
    }
}

impl KmpEntry for Ktpt {
    const FOURCC: [u8; 4] = *b"KTPT";
}
