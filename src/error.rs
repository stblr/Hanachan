use std::io;

#[derive(Debug)]
pub enum Error {
    Parsing,
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
