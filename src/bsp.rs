use crate::take::{self, Take, TakeFromSlice};
use crate::vec3::Vec3;

#[derive(Clone, Debug)]
pub struct Bsp {
    initial_pos_y: f32,
    hitboxes: [Option<Hitbox>; 16],
    cuboids: [Vec3; 2],
    rot_factor: f32,
    wheels: [Wheel; 2],
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
struct Hitbox {
    center: Vec3,
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

        let center = slice.take()?;
        let radius = slice.take()?;
        let walls_only = match slice.take::<u16>()? {
            0 => false,
            1 => true,
            _ => return Err(take::Error {}),
        };
        let _wheel_idx = slice.take::<u16>()?;

        Ok(Some(Hitbox {
            center,
            radius,
            walls_only,
        }))
    }
}

#[derive(Clone, Copy, Debug)]
struct Wheel {
    distance_suspension: f32,
    speed_suspension: f32,
    slack_y: f32,
    topmost_pos: Vec3,
    wheel_radius: f32,
    hitbox_radius: f32,
}

impl TakeFromSlice for Wheel {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Wheel, take::Error> {
        if slice.take::<u16>()? != 1 {
            return Err(take::Error {});
        }
        let _padding = slice.take::<u16>()?;

        let distance_suspension = slice.take()?;
        let speed_suspension = slice.take()?;
        let slack_y = slice.take()?;
        let _rot_x = slice.take::<f32>()?;
        let topmost_pos = slice.take()?;
        let wheel_radius = slice.take()?;
        let hitbox_radius = slice.take()?;
        let _unknown = slice.take::<u32>()?;

        Ok(Wheel {
            distance_suspension,
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
