use crate::fs::{Error, KmpEntry, Parse, SliceRefExt};
use crate::geom::Vec2;

#[derive(Clone, Copy, Debug)]
pub struct Ckpt {
    pub left: Vec2,
    pub right: Vec2,
    pub jgpt_idx: u8,
    pub kind: Kind,
    pub prev_idx: Option<u8>,
    pub next_idx: Option<u8>,
}

impl Parse for Ckpt {
    fn parse(input: &mut &[u8]) -> Result<Ckpt, Error> {
        let left = input.take()?;
        let right = input.take()?;
        let jgpt_idx = input.take()?;
        let kind = input.take()?;
        let prev_idx = Some(input.take()?).filter(|prev_idx| *prev_idx != 255);
        let next_idx = Some(input.take()?).filter(|next_idx| *next_idx != 255);

        Ok(Ckpt {
            left,
            right,
            jgpt_idx,
            kind,
            prev_idx,
            next_idx,
        })
    }
}

impl KmpEntry for Ckpt {
    const FOURCC: [u8; 4] = *b"CKPT";
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    FinishLine,
    Key { idx: u8 },
    Normal,
}

impl Parse for Kind {
    fn parse(input: &mut &[u8]) -> Result<Kind, Error> {
        input.take().map(|val| match val {
            0 => Kind::FinishLine,
            255 => Kind::Normal,
            idx => Kind::Key { idx },
        })
    }
}
