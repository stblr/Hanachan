use crate::geom::Vec3;
use crate::take::{self, Take, TakeFromSlice};

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

impl TakeFromSlice for Handle {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Handle, take::Error> {
        Ok(Handle {
            pos: slice.take()?,
            angles: slice.take::<Vec3>()?.to_radians(),
        })
    }
}
