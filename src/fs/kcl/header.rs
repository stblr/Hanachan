use crate::fs::{Error, Parse, ResultExt, SliceRefExt};
use crate::geom::Vec3;

#[derive(Clone, Debug)]
pub struct Header {
    pub poss_offset: u32,
    pub nors_offset: u32,
    pub tris_offset: u32,
    pub octree_offset: u32,
    pub thickness: f32,
    pub origin: Vec3,
    pub x_mask: u32,
    pub y_mask: u32,
    pub z_mask: u32,
    pub shift: u32,
    pub y_shift: u32,
    pub z_shift: u32,
    pub root_node_count: u32,
    pub max_radius: f32,
}

impl Parse for Header {
    fn parse(input: &mut &[u8]) -> Result<Header, Error> {
        let poss_offset = input
            .take::<u32>()
            .filter(|poss_offset| *poss_offset == 0x3c)?;
        let nors_offset = input.take()?;
        let tris_offset = input.take::<u32>()?.checked_add(0x10).ok_or(Error {})?;
        let octree_offset = input.take()?;

        let thickness = input.take()?;
        let origin = input.take()?;

        let x_mask = input
            .take::<u32>()
            .filter(|x_mask| x_mask.trailing_zeros() == x_mask.count_zeros())?;
        let y_mask = input
            .take::<u32>()
            .filter(|y_mask| y_mask.trailing_zeros() == y_mask.count_zeros())?;
        let z_mask = input
            .take::<u32>()
            .filter(|z_mask| z_mask.trailing_zeros() == z_mask.count_zeros())?;
        let shift = input.take()?;
        let y_shift = input.take()?;
        let z_shift = input.take::<u32>()?;

        let root_bits_x = x_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        if root_bits_x != y_shift {
            return Err(Error {});
        }
        let root_bits_y = y_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        if root_bits_y != z_shift.checked_sub(y_shift).ok_or(Error {})? {
            return Err(Error {});
        }
        let root_bits_z = z_mask.trailing_zeros().checked_sub(shift).ok_or(Error {})?;
        let root_bits = root_bits_x + root_bits_y + root_bits_z;
        let root_node_count = 1u32.checked_shl(root_bits).ok_or(Error {})?;

        let max_radius = input.take()?;

        Ok(Header {
            poss_offset,
            nors_offset,
            tris_offset,
            octree_offset,
            thickness,
            origin,
            x_mask,
            y_mask,
            z_mask,
            shift,
            y_shift,
            z_shift,
            root_node_count,
            max_radius,
        })
    }
}
