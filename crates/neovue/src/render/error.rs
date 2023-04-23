use std::error::Error;
use std::fmt::Display;
use std::io;

#[derive(Debug)]
#[non_exhaustive]
pub enum RenderError {
    Io(io::Error),
    JsonSerialization(serde_json::Error),
}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::Io(inner) => Display::fmt(inner, f),
            RenderError::JsonSerialization(inner) => Display::fmt(inner, f),
        }
    }
}

impl Error for RenderError {}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        RenderError::Io(error)
    }
}

impl From<serde_json::Error> for RenderError {
    fn from(error: serde_json::Error) -> Self {
        RenderError::JsonSerialization(error)
    }
}
