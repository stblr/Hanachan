use crate::fs::{Error, Parse, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Handle {
    pos: Vec3,
    angles: Vec3,
}

impl Handle {
    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn angles(&self) -> Vec3 {
        self.angles
    }
}

impl Parse for Handle {
    fn parse(input: &mut &[u8]) -> Result<Handle, Error> {
        Ok(Handle {
            pos: input.take()?,
            angles: input.take::<Vec3>()?.to_radians(),
        })
    }
}
