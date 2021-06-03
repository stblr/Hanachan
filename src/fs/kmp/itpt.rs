use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Itpt {
    pub pos: Vec3,
}

impl Parse for Itpt {
    fn parse(input: &mut &[u8]) -> Result<Itpt, Error> {
        let pos = input.take()?;

        input.skip(0x14 - 0xc)?;

        Ok(Itpt { pos })
    }
}

impl KmpEntry for Itpt {
    const FOURCC: [u8; 4] = *b"ITPT";
}
