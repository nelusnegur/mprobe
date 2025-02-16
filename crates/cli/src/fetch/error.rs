use std::io;

use reqwest::StatusCode;

pub(crate) type Result<T> = std::result::Result<T, FetchError>;

#[derive(Debug)]
pub(crate) enum FetchError {
    Http(reqwest::Error),
    DigestAuth(String),
    Response {
        status_code: StatusCode,
        message: String,
    },
    Io(io::Error),
    Job(JobErrorStatus),
}

#[derive(Debug)]
pub(crate) enum JobErrorStatus {
    Failure,
    MarkedForExpiry,
    Expired,
}

impl From<reqwest::Error> for FetchError {
    fn from(error: reqwest::Error) -> Self {
        FetchError::Http(error)
    }
}

impl From<io::Error> for FetchError {
    fn from(error: io::Error) -> Self {
        FetchError::Io(error)
    }
}

impl From<reqwest::header::ToStrError> for FetchError {
    fn from(error: reqwest::header::ToStrError) -> Self {
        FetchError::DigestAuth(format!("Digest authentication error: {error}"))
    }
}

impl From<digest_auth::Error> for FetchError {
    fn from(error: digest_auth::Error) -> Self {
        FetchError::DigestAuth(format!("Digest authentication error: {error}"))
    }
}
