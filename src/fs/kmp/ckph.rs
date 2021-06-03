use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};

use super::GroupIdcs;

#[derive(Clone, Debug)]
pub struct Ckph {
    pub start: u8,
    pub len: u8,
    pub prev_group_idcs: Vec<u8>,
    pub next_group_idcs: Vec<u8>,
}

impl Parse for Ckph {
    fn parse(input: &mut &[u8]) -> Result<Ckph, Error> {
        let start = input.take()?;
        let len = input.take()?;
        let prev_group_idcs = input.take::<GroupIdcs>()?.into();
        let next_group_idcs = input.take::<GroupIdcs>()?.into();

        input.skip(0x2)?;

        Ok(Ckph {
            start,
            len,
            prev_group_idcs,
            next_group_idcs,
        })
    }
}

impl KmpEntry for Ckph {
    const FOURCC: [u8; 4] = *b"CKPH";
}
