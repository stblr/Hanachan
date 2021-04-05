use core::iter;

use std::fs;
use std::path::Path;

use crate::error;
use crate::player::Params;
use crate::slice_ext::SliceExt;
use crate::take::{self, Bits, Take, TakeFromSlice};
use crate::yaz;

#[derive(Clone, Debug)]
pub struct Rkg {
    header: Header,
    frames: Vec<Frame>,
    ctgp_footer: Option<CtgpFooter>,
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
        let (compressed, mut input) = input.try_split_at(compressed_size).ok_or(Error {})?;
        let decompressed = yaz::decompress(&compressed)?;
        let frames = Rkg::parse_frames(&decompressed)?;

        let _crc32 = input.take::<u32>()?;

        let ctgp_footer = (!input.is_empty())
            .then(|| CtgpFooter::parse(input))
            .transpose()?;

        Ok(Rkg {
            header,
            frames,
            ctgp_footer,
        })
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

    pub fn accelerate(&self, frame: u32) -> bool {
        match frame.checked_sub(172) {
            Some(frame) => self
                .frames
                .get(frame as usize)
                .filter(|frame| frame.accelerate)
                .is_some(),
            None => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    time: Time,
    track: u8,
    params: Params,
    year: u16,
    month: u8,
    day: u8,
    controller: u8,
    compressed: bool,
    lap_count: u8,
    lap_times: Vec<Time>,
}

impl Header {
    fn parse(mut input: &[u8]) -> Result<Header, Error> {
        let fourcc = input.take::<u32>()?;
        if fourcc != u32::from_be_bytes(*b"RKGD") {
            return Err(Error {});
        }

        let time = input.take::<Time>()?;

        let mut bits = Bits::new(input);
        let track = bits.take_u8(6)?;
        let _padding = bits.take_u8(2)?;
        if track >= 0x20 {
            return Err(Error {});
        }

        let vehicle_id = bits.take_u8(6)?;
        let character_id = bits.take_u8(6)?;
        let params = Params::try_from_raw(vehicle_id, character_id).ok_or(take::Error {})?;

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

        let _padding = bits.take_u8(2)?;
        let _ghost_type = bits.take_u8(7)?;
        let _automatic = bits.take_bool()?;
        let _padding = bits.take_u8(1)?;

        let mut input = bits.try_into_inner().unwrap();
        let _decompressed_size = input.take::<u16>()?;

        let lap_count = input.take::<u8>()?;
        if lap_count > 9 {
            return Err(Error {});
        }
        let lap_times = iter::repeat_with(|| input.take::<Time>())
            .take(lap_count as usize)
            .collect::<Result<Vec<Time>, take::Error>>()?;
        for _ in lap_count..9 {
            for _ in 0..3 {
                let _unused = input.take::<u8>()?;
            }
        }

        for _ in 0..8 {
            let _padding = input.take::<u8>()?;
        }

        Ok(Header {
            time,
            track,
            params,
            year,
            month,
            day,
            controller,
            compressed,
            lap_count,
            lap_times,
        })
    }

    pub fn params(&self) -> &Params {
        &self.params
    }
}

#[derive(Clone, Copy, Debug)]
struct Time {
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
}

impl TakeFromSlice for Time {
    fn take_from_slice(slice: &mut &[u8]) -> Result<Time, take::Error> {
        let mut bits = Bits::new(slice);
        let minutes = bits.take_u8(7)?;
        let seconds = bits.take_u8(7)?;
        let milliseconds = bits.take_u16(10)?;
        *slice = bits.try_into_inner().unwrap();
        if minutes >= 6 || seconds >= 60 || milliseconds >= 1000 {
            Err(take::Error {})
        } else {
            Ok(Time {
                minutes,
                seconds,
                milliseconds,
            })
        }
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
struct CtgpFooter {
    track_sha1: [u32; 5],
    player_id: u64,
    true_time: f32,
    ctgp_version: u32,
    lap_dubious_intersections: [bool; 10],
    lap_true_times: [f32; 10],
    rtc_end: u64,
    rtc_start: u64,
    rtc_paused: u64,
    my_stuff_enabled: bool,
    my_stuff_used: bool,
    usb_gcn_enabled: bool,
    dubious_intersection: bool,
    mushrooms: [u8; 3],
    shortcut_definition_version: u8,
    cannon: bool,
    oob: bool,
    slowdown: bool,
    rapidfire: bool,
    dubious: bool,
    replaced_mii_data: bool,
    replaced_name: bool,
    respawn: bool,
    category: u8,
}

impl CtgpFooter {
    fn parse(mut input: &[u8]) -> Result<CtgpFooter, Error> {
        input.skip(0x48)?;

        let mut track_sha1 = [0; 5];
        for i in 0..5 {
            track_sha1[i] = input.take()?;
        }

        let player_id = input.take()?;
        let true_time = input.take()?;
        let ctgp_version = input.take()?;

        let mut bits = Bits::new(input);
        let mut lap_dubious_intersections = [false; 10];
        for i in 0..10 {
            lap_dubious_intersections[i] = bits.take_bool()?;
        }
        let _padding = bits.take_u8(6)?;

        let mut input = bits.try_into_inner().unwrap();
        input.skip(0x12)?;
        let mut lap_true_times = [0.0; 10];
        for i in 0..10 {
            lap_true_times[i] = input.take()?;
        }
        lap_true_times.reverse();

        let rtc_end = input.take()?;
        let rtc_start = input.take()?;
        let rtc_paused = input.take()?;

        let mut bits = Bits::new(input);
        let _padding = bits.take_u8(4)?;
        let my_stuff_enabled = bits.take_bool()?;
        let my_stuff_used = bits.take_bool()?;
        let usb_gcn_enabled = bits.take_bool()?;
        let dubious_intersection = bits.take_bool()?;

        let mut input = bits.try_into_inner().unwrap();
        let mut mushrooms = [input.take()?, input.take()?, input.take()?];
        mushrooms.reverse();
        let shortcut_definition_version = input.take()?;

        let mut bits = Bits::new(input);
        let cannon = bits.take_bool()?;
        let oob = bits.take_bool()?;
        let slowdown = bits.take_bool()?;
        let rapidfire = bits.take_bool()?;
        let dubious = bits.take_bool()?;
        let replaced_mii_data = bits.take_bool()?;
        let replaced_name = bits.take_bool()?;
        let respawn = bits.take_bool()?;

        let mut input = bits.try_into_inner().unwrap();
        let category = input.take()?;

        Ok(CtgpFooter {
            track_sha1,
            player_id,
            true_time,
            ctgp_version,
            lap_dubious_intersections,
            lap_true_times,
            rtc_end,
            rtc_start,
            rtc_paused,
            my_stuff_enabled,
            my_stuff_used,
            usb_gcn_enabled,
            dubious_intersection,
            mushrooms,
            shortcut_definition_version,
            cannon,
            oob,
            slowdown,
            rapidfire,
            dubious,
            replaced_mii_data,
            replaced_name,
            respawn,
            category,
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
