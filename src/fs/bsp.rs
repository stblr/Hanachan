use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Bsp {
    pub initial_pos_y: f32,
    pub hitboxes: [Option<Hitbox>; 16],
    pub cuboids: [Vec3; 2],
    pub rot_factor: f32,
    pub wheels: [Wheel; 2],
}

impl Parse for Bsp {
    fn parse(input: &mut &[u8]) -> Result<Bsp, Error> {
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

impl Parse for Option<Hitbox> {
    fn parse(input: &mut &[u8]) -> Result<Option<Hitbox>, Error> {
        match input.take::<u16>()? {
            0 => {
                input.skip(0x16)?;
                return Ok(None);
            }
            1 => (),
            _ => return Err(Error {}),
        }
        let _padding = input.take::<u16>()?;

        let pos = input.take()?;
        let radius = input.take()?;
        let walls_only = match input.take::<u16>()? {
            0 => false,
            1 => true,
            _ => return Err(Error {}),
        };
        let _wheel_idx = input.take::<u16>()?;

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

impl Parse for Wheel {
    fn parse(input: &mut &[u8]) -> Result<Wheel, Error> {
        input.take::<u16>().filter(|enable| *enable == 1)?;
        let _padding = input.take::<u16>()?;

        let dist_suspension = input.take()?;
        let speed_suspension = input.take()?;
        let slack_y = input.take()?;
        let topmost_pos = input.take()?;
        let _rot_x = input.take::<f32>()?;
        let wheel_radius = input.take()?;
        let hitbox_radius = input.take()?;
        let _unknown = input.take::<u32>()?;

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
