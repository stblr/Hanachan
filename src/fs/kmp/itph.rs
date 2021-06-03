use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};

use super::GroupIdcs;

#[derive(Clone, Debug)]
pub struct Itph {
    pub start: u8,
    pub len: u8,
    pub prev_group_idcs: Vec<u8>,
    pub next_group_idcs: Vec<u8>,
}

impl Parse for Itph {
    fn parse(input: &mut &[u8]) -> Result<Itph, Error> {
        let start = input.take()?;
        let len = input.take()?;
        let prev_group_idcs = input.take::<GroupIdcs>()?.into();
        let next_group_idcs = input.take::<GroupIdcs>()?.into();

        input.skip(0x2)?;

        Ok(Itph {
            start,
            len,
            prev_group_idcs,
            next_group_idcs,
        })
    }
}

impl KmpEntry for Itph {
    const FOURCC: [u8; 4] = *b"ITPH";
}
