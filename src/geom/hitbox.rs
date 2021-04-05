use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pos: Vec3,
    radius: f32,
}

impl Hitbox {
    pub fn new(pos: Vec3, radius: f32) -> Hitbox {
        Hitbox { pos, radius }
    }

    pub fn check_collision(&self) -> Option<Vec3> {
        // TODO use Kcl
        let dist = 1000.0 - self.pos.y + self.radius;
        if dist <= 0.0 {
            None
        } else {
            Some(dist * Vec3::UP)
        }
    }
}
