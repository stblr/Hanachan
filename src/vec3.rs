use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl TakeFromSlice for Vec3 {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Vec3, take::Error> {
        Ok(Vec3 {
            x: slice.take::<f32>()?,
            y: slice.take::<f32>()?,
            z: slice.take::<f32>()?,
        })
    }
}
