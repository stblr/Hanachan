use core::iter;

use std::fs;
use std::path::Path;

use crate::error;
use crate::slice_ext::SliceExt;
use crate::take::{self, Bits, Take};
use crate::yaz;

#[derive(Clone, Debug)]
pub struct Rkg {
    header: Header,
    frames: Vec<Frame>,
}

impl Rkg {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Rkg, error::Error> {
        let input = fs::read(path)?;
        Rkg::parse(&input).map_err(Into::into)
    }

    fn parse(input: &[u8]) -> Result<Rkg, Error> {
        let (header_input, mut input) = input.try_split_at(0x88).ok_or(Error {})?;
        let header = Header::parse(header_input)?;

        let compressed_size = input.take::<u32>()? as usize;
        let (compressed, _input) = input.try_split_at(compressed_size).ok_or(Error {})?;
        let decompressed = yaz::decompress(&compressed)?;
        let frames = Rkg::parse_frames(&decompressed)?;

        Ok(Rkg { header, frames })
    }

    fn parse_frames(mut input: &[u8]) -> Result<Vec<Frame>, Error> {
        let face_button_input_count = input.take::<u16>()? as usize;
        let direction_input_count = input.take::<u16>()? as usize;
        let trick_input_count = input.take::<u16>()? as usize;
        let _padding_0x6 = input.take::<u16>()?;

        let (mut face_button_inputs, input) = input
            .try_split_at(2 * face_button_input_count)
            .ok_or(Error {})?;
        let (mut direction_inputs, input) = input
            .try_split_at(2 * direction_input_count)
            .ok_or(Error {})?;
        let (mut trick_inputs, input) =
            input.try_split_at(2 * trick_input_count).ok_or(Error {})?;
        if input.len() != 0 {
            return Err(Error {});
        }

        let mut face_button_iter = iter::from_fn(|| {
            let input = face_button_inputs.take::<u8>().ok()?;
            let frame_count = face_button_inputs.take::<u8>().ok()? as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let mut direction_iter = iter::from_fn(|| {
            let input = direction_inputs.take::<u8>().ok()?;
            let frame_count = direction_inputs.take::<u8>().ok()? as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let mut trick_iter = iter::from_fn(|| {
            let val = trick_inputs.take::<u16>().ok()?;
            let input = (val >> 12) as u8;
            let frame_count = (val & 0xfff) as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let frames = face_button_iter
            .by_ref()
            .zip(direction_iter.by_ref())
            .zip(trick_iter.by_ref())
            .map(|((face_button, direction), trick)| Frame::new(face_button, direction, trick))
            .collect::<Result<Vec<Frame>, Error>>()?;

        if face_button_iter.next().is_some()
            || direction_iter.next().is_some()
            || trick_iter.next().is_some()
        {
            Err(Error {})
        } else {
            Ok(frames)
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Header {
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
    track: u8,
    vehicle: u8,
    character: u8,
    year: u16,
    month: u8,
    day: u8,
    controller: u8,
    compressed: bool,
}

impl Header {
    fn parse(mut input: &[u8]) -> Result<Header, Error> {
        let fourcc = input.take::<u32>()?;
        if fourcc != u32::from_be_bytes(*b"RKGD") {
            return Err(Error {});
        }

        let mut bits = Bits::new(input);
        let minutes = bits.take_u8(7)?;
        let seconds = bits.take_u8(7)?;
        let milliseconds = bits.take_u16(10)?;
        if minutes >= 6 || seconds >= 60 || milliseconds >= 1000 {
            return Err(Error {});
        }

        let track = bits.take_u8(6)?;
        let _padding = bits.take_u8(2)?;
        if track >= 0x20 {
            return Err(Error {});
        }

        let vehicle = bits.take_u8(6)?;
        if vehicle >= 0x24 {
            return Err(Error {});
        }
        let character = bits.take_u8(6)?;
        if character >= 0x18 {
            // TODO support Miis
            return Err(Error {});
        }

        let year = 2000 + bits.take_u8(7)? as u16;
        let month = bits.take_u8(4)?;
        let day = bits.take_u8(5)?;
        // TODO validate

        let controller = bits.take_u8(4)?;
        if controller >= 4 {
            return Err(Error {});
        }

        let _padding = bits.take_u8(4)?;
        let compressed = bits.take_bool()?;
        if !compressed {
            return Err(Error {});
        }

        Ok(Header {
            minutes,
            seconds,
            milliseconds,
            track,
            vehicle,
            character,
            year,
            month,
            day,
            controller,
            compressed,
        })
    }
}

#[derive(Clone, Copy, Debug)]
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
    fn new(face_button: u8, direction: u8, trick: u8) -> Result<Frame, Error> {
        if face_button >> 4 != 0 {
            return Err(Error {});
        }
        let accelerate = face_button & 1 != 0;
        let brake = face_button >> 1 & 1 != 0;
        let use_item = face_button >> 2 & 1 != 0;
        let drift = face_button >> 3 & 1 != 0;

        let stick_x = direction >> 4;
        let stick_y = direction & 0xf;
        if stick_x == 15 || stick_y == 15 {
            return Err(Error {});
        }

        if trick > 4 {
            return Err(Error {});
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
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Error {}

impl From<take::Error> for Error {
    fn from(_: take::Error) -> Error {
        Error {}
    }
}

impl From<yaz::Error> for Error {
    fn from(_: yaz::Error) -> Error {
        Error {}
    }
}
