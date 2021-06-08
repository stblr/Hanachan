use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pub pos: Vec3,
    pub last_pos: Option<Vec3>,
    pub radius: f32,
    pub flags: u32,
}

impl Hitbox {
    pub fn new(pos: Vec3, last_pos: Option<Vec3>, radius: f32, flags: u32) -> Hitbox {
        Hitbox { pos, last_pos, radius, flags }
    }

    pub fn update_pos(&mut self, pos: Vec3) {
        self.last_pos = Some(self.pos);
        self.pos = pos;
    }
}
