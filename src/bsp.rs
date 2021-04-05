use crate::geom::Vec3;
use crate::take::{self, Take, TakeFromSlice};

#[derive(Clone, Copy, Debug)]
pub struct Bsp {
    pub initial_pos_y: f32,
    pub hitboxes: [Option<Hitbox>; 16],
    pub cuboids: [Vec3; 2],
    pub rot_factor: f32,
    pub wheels: [Wheel; 2],
}

impl Bsp {
    pub fn parse(mut input: &[u8]) -> Result<Bsp, Error> {
        let initial_pos_y = input.take()?;
        let mut hitboxes = [None; 16];
        for i in 0..16 {
            hitboxes[i] = input.take()?;
        }
        let cuboids = [input.take()?, input.take()?];
        let rot_factor = input.take()?;
        let _unknown: f32 = input.take()?;
        let wheels = [input.take()?, input.take()?];

        Ok(Bsp {
            initial_pos_y,
            hitboxes,
            cuboids,
            rot_factor,
            wheels,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    pos: Vec3,
    radius: f32,
    walls_only: bool,
}

impl TakeFromSlice for Option<Hitbox> {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Option<Hitbox>, take::Error> {
        match slice.take::<u16>()? {
            0 => {
                slice.skip(0x16)?;
                return Ok(None);
            }
            1 => (),
            _ => return Err(take::Error {}),
        }
        let _padding = slice.take::<u16>()?;

        let pos = slice.take()?;
        let radius = slice.take()?;
        let walls_only = match slice.take::<u16>()? {
            0 => false,
            1 => true,
            _ => return Err(take::Error {}),
        };
        let _wheel_idx = slice.take::<u16>()?;

        Ok(Some(Hitbox {
            pos,
            radius,
            walls_only,
        }))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Wheel {
    pub dist_suspension: f32,
    pub speed_suspension: f32,
    pub slack_y: f32,
    pub topmost_pos: Vec3,
    pub wheel_radius: f32,
    pub hitbox_radius: f32,
}

impl Wheel {
    pub fn mirror_x_pos(&mut self) {
        self.topmost_pos.x *= -1.0;
    }
}

impl TakeFromSlice for Wheel {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Wheel, take::Error> {
        if slice.take::<u16>()? != 1 {
            return Err(take::Error {});
        }
        let _padding = slice.take::<u16>()?;

        let dist_suspension = slice.take()?;
        let speed_suspension = slice.take()?;
        let slack_y = slice.take()?;
        let topmost_pos = slice.take()?;
        let _rot_x = slice.take::<f32>()?;
        let wheel_radius = slice.take()?;
        let hitbox_radius = slice.take()?;
        let _unknown = slice.take::<u32>()?;

        Ok(Wheel {
            dist_suspension,
            speed_suspension,
            slack_y,
            topmost_pos,
            wheel_radius,
            hitbox_radius,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}

impl From<take::Error> for Error {
    fn from(_: take::Error) -> Error {
        Error {}
    }
}
