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
