use std::fs;
use std::iter;
use std::path::Path;

use crate::geom::{Quat, Vec3};
use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Debug)]
pub struct Rkrd {
    frames: Vec<Frame>,
}

impl Rkrd {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Rkrd, take::Error> {
        let mut input: &[u8] = &fs::read(path).map_err(|_| take::Error {})?;
        input.take()
    }

    pub fn frames(&self) -> &Vec<Frame> {
        &self.frames
    }
}

impl TakeFromSlice for Rkrd {
    fn take_from_slice(input: &mut &[u8]) -> Result<Rkrd, take::Error> {
        let fourcc = input.take::<u32>()?;
        if fourcc != u32::from_be_bytes(*b"RKRD") {
            return Err(take::Error {});
        }

        let version = input.take::<u32>()?;
        if version != 0 {
            return Err(take::Error {});
        }

        let frames = iter::from_fn(|| (!input.is_empty()).then(|| input.take()))
            .collect::<Result<_, _>>()?;

        Ok(Rkrd { frames })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub dir: Vec3,
    pub pos: Vec3,
    pub vel0: Vec3,
    pub speed1: f32,
    pub vel: Vec3,
    pub rot_vec0: Vec3,
    pub rot_vec2: Vec3,
    pub rot0: Quat,
    pub rot1: Quat,
}

impl TakeFromSlice for Frame {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Frame, take::Error> {
        Ok(Frame {
            dir: slice.take()?,
            pos: slice.take()?,
            vel0: slice.take()?,
            speed1: slice.take()?,
            vel: slice.take()?,
            rot_vec0: slice.take()?,
            rot_vec2: slice.take()?,
            rot0: slice.take()?,
            rot1: slice.take()?,
        })
    }
}
