use std::io::Error as IoError;

use crate::fs::Error as ParsingError;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Parsing,
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<ParsingError> for Error {
    fn from(_: ParsingError) -> Error {
        Error::Parsing
    }
}
