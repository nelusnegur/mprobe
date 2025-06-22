//! Defines the `Error` and `Result` types that this crate uses.

use std::error::Error;
use std::fmt::Display;
use std::io::Error as IoError;

use tinytemplate::error::Error as TinyTemplateError;

/// The result type that uses [VisError] as the error type.
pub type Result<T> = std::result::Result<T, VisError>;

/// The error type for generating a visualization report
/// of diagnostic metrics.
#[derive(Debug)]
pub enum VisError {
    /// A [std::io::Error] encountered while generating files
    /// for the data visualization.
    Io(IoError),

    /// A [tinytemplate::error::Error] encountered while parsing rendering
    /// a template file.
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
