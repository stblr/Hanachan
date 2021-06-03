use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Enpt {
    pub pos: Vec3,
}

impl Parse for Enpt {
    fn parse(input: &mut &[u8]) -> Result<Enpt, Error> {
        let pos = input.take()?;

        input.skip(0x14 - 0xc)?;

        Ok(Enpt { pos })
    }
}

impl KmpEntry for Enpt {
    const FOURCC: [u8; 4] = *b"ENPT";
}
