use crate::error::Error;
use crate::view::View;

pub fn decompress(view: &View) -> Result<Vec<u8>, Error> {
    if view.len() < 0x10 {
        return Err(Error::Parsing);
    }

    let fourcc = &view.inner()[0x00..0x04];
    if fourcc != b"Yaz0" && fourcc != b"Yaz1" {
        return Err(Error::Parsing);
    }

    let len = view.index_u32(0x04) as usize;

    let view = View::new(&view.inner()[0x10..]);
    let mut stream = view.stream();
    let mut group_header = stream.next_u8().ok_or(Error::Parsing)?;
    let mut data = Vec::new();
    for group_shift in (0..8).rev().cycle() {
        if group_header >> group_shift & 1 != 0 {
            data.push(stream.next_u8().ok_or(Error::Parsing)?);
        } else {
            let val = stream.next_u16().ok_or(Error::Parsing)?;
            let ref_start = data.len().checked_sub(((val & 0xfff) + 1) as usize).ok_or(Error::Parsing)?;
            let ref_size = match val >> 12 {
                0 => stream.next_u8().ok_or(Error::Parsing)? as usize + 18,
                ref_size => ref_size as usize + 2,
            };
            for offset in 0..ref_size {
                data.push(data[ref_start + offset]);
            }
        }

        if data.len() > len {
            return Err(Error::Parsing);
        } else if data.len() == len {
            return Ok(data);
        }

        if group_shift == 0 {
            group_header = stream.next_u8().ok_or(Error::Parsing)?;
        }
    }

    unreachable!()
}
