use std::error::Error;
use std::fmt::Display;
use std::io;
use std::sync::Arc;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum RenderError {
    IO(Arc<io::Error>),
}

impl Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::IO(inner) => Display::fmt(inner, f),
        }
    }
}

impl Error for RenderError {}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        RenderError::IO(Arc::new(error))
    }
}
