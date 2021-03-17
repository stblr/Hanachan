use std::io;

use crate::u8;
use crate::yaz;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parsing, // TODO remove
    U8,
    Yaz,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}

impl From<u8::Error> for Error {
    fn from(_: u8::Error) -> Error {
        Error::U8
    }
}

impl From<yaz::Error> for Error {
    fn from(_: yaz::Error) -> Error {
        Error::Yaz
    }
}
