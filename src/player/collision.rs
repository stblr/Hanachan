use crate::geom::Vec3;

#[derive(Clone, Debug)]
pub struct Collision {
    pub floor_nor: Vec3,
    pub speed_factor: f32,
    pub rot_factor: f32,
}
