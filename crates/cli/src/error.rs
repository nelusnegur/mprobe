use std::fmt::Display;

use crate::fetch::error::FetchError;

#[derive(Debug)]
pub(crate) enum CliError {
    Fetch(FetchError),
    Path(String),
}

impl From<FetchError> for CliError {
    fn from(error: FetchError) -> Self {
        CliError::Fetch(error)
    }
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cli_error = "CLI error:";

        match self {
            CliError::Fetch(error) => write!(f, "{cli_error} {error}"),
            CliError::Path(error) => write!(f, "{cli_error} {error}"),
        }
    }
}
