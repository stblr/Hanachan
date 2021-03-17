use crate::take::{self, Take};

pub fn decompress(mut input: &[u8]) -> Result<Vec<u8>, Error> {
    let fourcc = input.take::<u32>()?;
    let yaz0 = u32::from_be_bytes(*b"Yaz0");
    let yaz1 = u32::from_be_bytes(*b"Yaz1");
    if fourcc != yaz0 && fourcc != yaz1 {
        return Err(Error {});
    }

    let len = input.take::<u32>()? as usize;

    let _reserved_0x8 = input.take::<u32>()?;
    let _reserved_0xc = input.take::<u32>()?;

    let mut group_header = input.take::<u8>()?;
    let mut output = Vec::new();
    for group_shift in (0..8).rev().cycle() {
        if group_header >> group_shift & 1 != 0 {
            output.push(input.take::<u8>()?);
        } else {
            let val = input.take::<u16>()?;
            let ref_start = output
                .len()
                .checked_sub(((val & 0xfff) + 1) as usize)
                .ok_or(Error {})?;
            let ref_size = match val >> 12 {
                0 => input.take::<u8>()? as usize + 18,
                ref_size => ref_size as usize + 2,
            };
            for offset in 0..ref_size {
                output.push(output[ref_start + offset]);
            }
        }

        if output.len() > len {
            return Err(Error {});
        } else if output.len() == len {
            return Ok(output);
        }

        if group_shift == 0 {
            group_header = input.take::<u8>()?;
        }
    }

    unreachable!()
}

pub struct Error {}

impl From<take::Error> for Error {
    fn from(_: take::Error) -> Error {
        Error {}
    }
}
