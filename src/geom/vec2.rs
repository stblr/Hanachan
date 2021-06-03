use crate::fs::{Error, Parse, SliceRefExt};

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Parse for Vec2 {
    fn parse(input: &mut &[u8]) -> Result<Vec2, Error> {
        Ok(Vec2::new(input.take()?, input.take()?))
    }
}
