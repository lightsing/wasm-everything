use Error::*;

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error)
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Bincode(e)
    }
}

