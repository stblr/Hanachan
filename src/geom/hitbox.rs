use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub pos: Vec3,
    pub has_last_pos: bool,
    pub radius: f32,
    pub flags: u32,
}

impl Hitbox {
    pub fn new(pos: Vec3, has_last_pos: bool, radius: f32, flags: u32) -> Hitbox {
        Hitbox { pos, has_last_pos, radius, flags }
    }
}
