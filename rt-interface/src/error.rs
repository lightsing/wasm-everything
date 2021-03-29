use Error::*;

#[derive(Debug)]
pub enum Error {
    SerdeJson(serde_json::Error)
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        SerdeJson(e)
    }
}

