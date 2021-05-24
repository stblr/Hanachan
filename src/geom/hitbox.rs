use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub pos: Vec3,
    pub radius: f32,
}

impl Hitbox {
    pub fn new(pos: Vec3, radius: f32) -> Hitbox {
        Hitbox { pos, radius }
    }
}
