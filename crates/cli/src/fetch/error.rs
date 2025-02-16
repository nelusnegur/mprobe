use std::fmt::Display;
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

impl Display for JobErrorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            JobErrorStatus::Failure => write!(f, "failure"),
            JobErrorStatus::MarkedForExpiry => write!(f, "marked for expiry"),
            JobErrorStatus::Expired => write!(f, "expired"),
        }
    }
}

impl Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fetch_error = "fetch error:";

        match self {
            FetchError::Http(error) => write!(f, "{fetch_error} HTTP request error: {error}"),
            FetchError::DigestAuth(error) => {
                write!(f, "{fetch_error} digest authentication error: {error}")
            }
            FetchError::Response {
                status_code,
                message,
            } => write!(
                f,
                "{fetch_error} HTTP response error: status = {status_code}, message = {message}"
            ),
            FetchError::Io(error) => {
                write!(f, "{fetch_error} downloading the archive failed: {error}")
            }
            FetchError::Job(status) => write!(
                f,
                "{fetch_error} the server job status: {status}; try creating another job"
            ),
        }
    }
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
