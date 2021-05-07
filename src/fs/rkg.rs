use std::iter;

use crate::fs::{yaz, Bits, Error, Parse, ResultExt, SliceExt, SliceRefExt};
use crate::player::Params;

#[derive(Clone, Debug)]
pub struct Rkg {
    header: Header,
    frames: Vec<Frame>,
    ctgp_footer: Option<CtgpFooter>,
}

impl Rkg {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn accelerate(&self, frame: u32) -> bool {
        frame
            .checked_sub(172)
            .and_then(|frame| self.frames.get(frame as usize))
            .map(|frame| frame.accelerate)
            .unwrap_or(false)
    }

    pub fn drift(&self, frame: u32) -> bool {
        frame
            .checked_sub(172)
            .and_then(|frame| self.frames.get(frame as usize))
            .map(|frame| frame.drift)
            .unwrap_or(false)
    }

    pub fn stick_x(&self, frame: u32) -> f32 {
        let discrete_stick_x = frame
            .checked_sub(172)
            .and_then(|frame| self.frames.get(frame as usize))
            .map(|frame| frame.stick_x)
            .unwrap_or(7);
        (discrete_stick_x as f32 - 7.0) / 7.0
    }

    pub fn stick_y(&self, frame: u32) -> f32 {
        let discrete_stick_y = frame
            .checked_sub(172)
            .and_then(|frame| self.frames.get(frame as usize))
            .map(|frame| frame.stick_y)
            .unwrap_or(7);
        (discrete_stick_y as f32 - 7.0) / 7.0
    }

    pub fn trick(&self, frame: u32) -> Option<Trick> {
        frame
            .checked_sub(172)
            .and_then(|frame| self.frames.get(frame as usize))
            .and_then(|frame| frame.trick)
    }
}

impl Parse for Rkg {
    fn parse(input: &mut &[u8]) -> Result<Rkg, Error> {
        let header = input.take()?;

        let compressed_size = input.take::<u32>()? as usize;
        let (compressed, mut input) = input.try_split_at(compressed_size).ok_or(Error {})?;
        let mut decompressed: &[u8] = &yaz::decompress(&compressed)?;
        let frames = decompressed.take()?;

        let _crc32 = input.take::<u32>()?;

        let ctgp_footer = (!input.is_empty()).then(|| input.take()).transpose()?;

        Ok(Rkg {
            header,
            frames,
            ctgp_footer,
        })
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
    pub fn params(&self) -> &Params {
        &self.params
    }
}

impl Parse for Header {
    fn parse(input: &mut &[u8]) -> Result<Header, Error> {
        input
            .take::<u32>()
            .filter(|fourcc| *fourcc == u32::from_be_bytes(*b"RKGD"))?;

        let time = input.take::<Time>()?;

        let mut bits = Bits::new(input);
        let track = bits.take_u8(6).filter(|track| *track < 32)?;
        let _padding = bits.take_u8(2)?;

        let vehicle_id = bits.take_u8(6)?;
        let character_id = bits.take_u8(6)?;
        let params = Params::try_from_raw(vehicle_id, character_id).ok_or(Error {})?;

        let year = 2000 + bits.take_u8(7)? as u16;
        let month = bits.take_u8(4)?;
        let day = bits.take_u8(5)?;
        // TODO validate

        let controller = bits.take_u8(4).filter(|controller| *controller < 4)?;

        let _padding = bits.take_u8(4)?;
        let compressed = bits.take_bool()?;
        if !compressed {
            return Err(Error {});
        }

        let _padding = bits.take_u8(2)?;
        let _ghost_type = bits.take_u8(7)?;
        let _automatic = bits.take_bool()?;
        let _padding = bits.take_u8(1)?;

        *input = bits.try_into_inner().unwrap();
        let _decompressed_size = input.take::<u16>()?;

        let lap_count = input.take::<u8>().filter(|lap_count| *lap_count <= 9)?;
        let lap_times = iter::repeat_with(|| input.take::<Time>())
            .take(lap_count as usize)
            .collect::<Result<_, _>>()?;
        for _ in lap_count..9 {
            for _ in 0..3 {
                let _unused = input.take::<u8>()?;
            }
        }

        for _ in 0..8 {
            let _padding = input.take::<u8>()?;
        }

        input.skip(0x88 - 0x34)?;

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
}

#[derive(Clone, Copy, Debug)]
struct Time {
    minutes: u8,
    seconds: u8,
    milliseconds: u16,
}

impl Parse for Time {
    fn parse(input: &mut &[u8]) -> Result<Time, Error> {
        let mut bits = Bits::new(input);
        let minutes = bits.take_u8(7).filter(|minutes| *minutes < 6)?;
        let seconds = bits.take_u8(7).filter(|seconds| *seconds < 60)?;
        let milliseconds = bits
            .take_u16(10)
            .filter(|milliseconds| *milliseconds < 1000)?;
        *input = bits.try_into_inner().unwrap();
        Ok(Time {
            minutes,
            seconds,
            milliseconds,
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
    trick: Option<Trick>,
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

        let trick = Trick::from_raw(trick);

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Trick {
    Up,
    Down,
    Left,
    Right,
}

impl Trick {
    pub fn from_raw(trick: u8) -> Option<Trick> {
        match trick {
            1 => Some(Trick::Up),
            2 => Some(Trick::Down),
            3 => Some(Trick::Left),
            4 => Some(Trick::Right),
            _ => None,
        }
    }
}

impl Parse for Vec<Frame> {
    fn parse(input: &mut &[u8]) -> Result<Vec<Frame>, Error> {
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

        let face_button_iter = iter::from_fn(|| {
            let input = face_button_inputs.take::<u8>().ok()?;
            let frame_count = face_button_inputs.take::<u8>().ok()? as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let direction_iter = iter::from_fn(|| {
            let input = direction_inputs.take::<u8>().ok()?;
            let frame_count = direction_inputs.take::<u8>().ok()? as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let trick_iter = iter::from_fn(|| {
            let val = trick_inputs.take::<u16>().ok()?;
            let input = (val >> 12) as u8;
            let frame_count = (val & 0xfff) as usize;
            Some(iter::repeat(input).take(frame_count))
        })
        .flatten();

        let frames = face_button_iter
            .zip(direction_iter)
            .zip(trick_iter)
            .map(|((face_button, direction), trick)| Frame::new(face_button, direction, trick))
            .collect::<Result<_, _>>()?;

        Ok(frames).filter(|_| {
            face_button_inputs.is_empty() && direction_inputs.is_empty() && trick_inputs.is_empty()
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

impl Parse for CtgpFooter {
    fn parse(input: &mut &[u8]) -> Result<CtgpFooter, Error> {
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

        *input = bits.try_into_inner().unwrap();
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

        *input = bits.try_into_inner().unwrap();
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

        *input = bits.try_into_inner().unwrap();
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
