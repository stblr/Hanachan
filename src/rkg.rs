use core::iter;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::Error;
use crate::view::View;
use crate::yaz;

pub struct Rkg {
    header: Header,
    frames: Vec<Frame>,
}

impl Rkg {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Rkg, Error> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let view = View::new(&buffer);
        Self::parse(&view)
    }

    fn parse(view: &View) -> Result<Rkg, Error> {
        let header = Header::parse(view)?;

        let compressed_len = view.get_u32(0x88).ok_or(Error::Parsing)? as usize;
        let compressed_view = View::new(&view.inner()[0x88 + 0x04..0x88 + 0x04 + compressed_len]);
        let decompressed = yaz::decompress(&compressed_view)?;
        let view = View::new(&decompressed);
        let frames = Frame::parse(&view)?;

        Ok(Rkg {
            header,
            frames,
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

#[derive(Debug)]
pub struct Header {
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
    track: u8,
    vehicle: u8,
    character: u8,
    compressed: bool,
}

impl Header {
    fn parse(view: &View) -> Result<Header, Error> {
        if view.len() < 0x88 || view.len() % 4 != 0 {
            return Err(Error::Parsing);
        }

        if &view.inner()[0x00..0x04] != b"RKGD" {
            return Err(Error::Parsing);
        }

        let minutes = view.index_u8(0x04) >> 1;
        let seconds = (view.index_u16(0x04) >> 2 & 0x7f) as u8;
        let milliseconds = view.index_u16(0x05) & 0x3ff;
        if minutes >= 6 || seconds >= 60 || milliseconds >= 1000 {
            return Err(Error::Parsing);
        }

        let track = view.index_u8(0x07) >> 2;
        if track >= 0x20 {
            return Err(Error::Parsing);
        }

        let vehicle = view.index_u8(0x08) >> 2;
        if vehicle >= 0x24 {
            return Err(Error::Parsing);
        }
        let character = (view.index_u16(0x08) >> 4 & 0x3f) as u8;
        if character >= 0x18 {
            // TODO support Miis
            return Err(Error::Parsing);
        }

        let compressed = (view.index_u8(0x0c) >> 3 & 1) != 0;
        if !compressed {
            return Err(Error::Parsing);
        }

        Ok(Header {
            minutes,
            seconds,
            milliseconds,
            track,
            vehicle,
            character,
            compressed,
        })
    }
}

#[derive(Debug)]
struct Frame {
    accelerate: bool,
    brake: bool,
    use_item: bool,
    drift: bool,
    stick_x: u8,
    stick_y: u8,
    trick: u8,
}

impl Frame {
    fn parse(view: &View) -> Result<Vec<Frame>, Error> {
        if view.len() < 8 {
            return Err(Error::Parsing);
        }

        let face_button_input_count = view.index_u16(0) as usize;
        let direction_input_count = view.index_u16(2) as usize;
        let trick_input_count = view.index_u16(4) as usize;
        let total_input_count = face_button_input_count + direction_input_count + trick_input_count;
        if total_input_count * 2 != view.len() - 8 {
            return Err(Error::Parsing);
        }

        let face_button_view = View::new(&view.inner()[8..]);
        let face_button_frame_count: u32 = (0..face_button_input_count)
            .map(|index| face_button_view.index_u8(2 * index + 1) as u32)
            .sum();

        let direction_view = View::new(&face_button_view.inner()[2 * face_button_input_count..]);
        let direction_frame_count: u32 = (0..direction_input_count)
            .map(|index| direction_view.index_u8(2 * index + 1) as u32)
            .sum();

        let trick_view = View::new(&direction_view.inner()[2 * direction_input_count..]);
        let trick_frame_count: u32 = (0..trick_input_count)
            .map(|index| (trick_view.index_u16(2 * index) & 0xfff) as u32)
            .sum();

        if face_button_frame_count != direction_frame_count ||
            face_button_frame_count != trick_frame_count {
            return Err(Error::Parsing);
        }

        let face_button_iter = (0..face_button_input_count)
            .flat_map(|index| {
                let val = face_button_view.index_u8(2 * index);
                let frame_count = face_button_view.index_u8(2 * index + 1);
                iter::repeat(val).take(frame_count as usize)
            });

        let direction_iter = (0..direction_input_count)
            .flat_map(|index| {
                let val = direction_view.index_u8(2 * index);
                let frame_count = direction_view.index_u8(2 * index + 1);
                iter::repeat(val).take(frame_count as usize)
            });

        let trick_iter = (0..trick_input_count)
            .flat_map(|index| {
                let val = trick_view.index_u8(2 * index) >> 4;
                let frame_count = trick_view.index_u16(2 * index) & 0xfff;
                iter::repeat(val).take(frame_count as usize)
            });

        face_button_iter.zip(direction_iter).zip(trick_iter)
            .map(|((face_button, direction), trick)| {
                if face_button >> 4 != 0 {
                    return Err(Error::Parsing);
                }
                let accelerate = face_button & 1 != 0;
                let brake = face_button >> 1 & 1 != 0;
                let use_item = face_button >> 2 & 1 != 0;
                let drift = face_button >> 3 & 1 != 0;

                let stick_x = direction >> 4;
                let stick_y = direction & 0xf;
                if stick_x == 15 || stick_y == 15 {
                    return Err(Error::Parsing);
                }

                if trick > 4 {
                    return Err(Error::Parsing);
                }

                Ok(Frame {
                    accelerate,
                    brake,
                    use_item,
                    drift,
                    stick_x,
                    stick_y,
                    trick,
                })
            })
            .collect()
    }
}
