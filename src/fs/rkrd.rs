use std::iter;

use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::geom::{Quat, Vec3};

#[derive(Clone, Debug)]
pub struct Rkrd {
    frames: Vec<Frame>,
}

impl Rkrd {
    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }
}

impl Parse for Rkrd {
    fn parse(input: &mut &[u8]) -> Result<Rkrd, Error> {
        input
            .take::<u32>()
            .filter(|fourcc| *fourcc == u32::from_be_bytes(*b"RKRD"))?;
        input.take::<u32>().filter(|version| *version == 2)?;

        let frames = iter::from_fn(|| (!input.is_empty()).then(|| input.take()))
            .collect::<Result<_, _>>()?;

        Ok(Rkrd { frames })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub floor_nor: Vec3,
    pub dir: Vec3,
    pub pos: Vec3,
    pub vel0: Vec3,
    pub speed1: f32,
    pub speed1_soft_limit: f32,
    pub vel2: Vec3,
    pub vel: Vec3,
    pub rot_vec0: Vec3,
    pub rot_vec2: Vec3,
    pub rot0: Quat,
    pub rot1: Quat,
    pub animation: u16,
    pub checkpoint_idx: u16,
}

impl Parse for Frame {
    fn parse(input: &mut &[u8]) -> Result<Frame, Error> {
        Ok(Frame {
            rot_vec2: input.take()?,
            speed1_soft_limit: input.take()?,
            speed1: input.take()?,
            floor_nor: input.take()?,
            dir: input.take()?,
            pos: input.take()?,
            vel0: input.take()?,
            rot_vec0: input.take()?,
            vel2: input.take()?,
            vel: input.take()?,
            rot0: input.take()?,
            rot1: input.take()?,
            animation: input.take()?,
            checkpoint_idx: input.take()?,
        })
    }
}
