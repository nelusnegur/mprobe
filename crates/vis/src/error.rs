use std::error::Error;
use std::fmt::Display;
use std::io::Error as IoError;

use tinytemplate::error::Error as TinyTemplateError;

pub type Result<T> = std::result::Result<T, VisError>;

#[derive(Debug)]
pub enum VisError {
    Io(IoError),
    TemplateError(TinyTemplateError),
}

impl Error for VisError {}

impl Display for VisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vis_error = "vis error:";

        match self {
            VisError::Io(error) => write!(f, "{vis_error} I/O error: {error}"),
            VisError::TemplateError(error) => write!(f, "{vis_error} template error: {error}"),
        }
    }
}

impl From<TinyTemplateError> for VisError {
    fn from(error: TinyTemplateError) -> Self {
        VisError::TemplateError(error)
    }
}

impl From<IoError> for VisError {
    fn from(error: IoError) -> Self {
        VisError::Io(error)
    }
}
