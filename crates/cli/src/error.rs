use std::error::Error;
use std::fmt::Display;

use mprobe_vis::error::VisError;

use crate::fetch::error::FetchError;

#[derive(Debug)]
pub(crate) enum CliError {
    Fetch(FetchError),
    View(VisError),
    Path(String),
}

impl Error for CliError {}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cli_error = "CLI error:";

        match self {
            CliError::Fetch(error) => write!(f, "{cli_error} {error}"),
            CliError::Path(error) => write!(f, "{cli_error} {error}"),
            CliError::View(error) => write!(f, "{cli_error} {error}"),
        }
    }
}

impl From<FetchError> for CliError {
    fn from(error: FetchError) -> Self {
        CliError::Fetch(error)
    }
}

impl From<VisError> for CliError {
    fn from(error: VisError) -> Self {
        CliError::View(error)
    }
}
