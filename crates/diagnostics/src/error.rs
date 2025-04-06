use std::convert::From;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::num::TryFromIntError;
use std::sync::Arc;

use bson::de;
use bson::document::ValueAccessError;

/// The error type for parsing diagnostic metrics.
///
/// Errors mostly originate from I/O read operations, BSON deserialization and field value accesses.
#[derive(Debug, Clone)]
pub enum MetricParseError {
    /// A [`std::io::Error`] encountered while reading metric chunks.
    Io(Arc<io::Error>),

    /// A [`bson::de::Error`] encountered while deserializing BSON documents.
    BsonDeserialzation(de::Error),

    /// A [`KeyAccessError`] encountered while accessing BSON fields.
    FieldAccess(KeyAccessError),

    /// Unknown document type.
    UnknownDocumentKind(i32),

    /// The metrics count from the reference document and the metrics count from samples do not
    /// match.
    MetricsCountMismatch,

    /// The metric timestamps for the given metric are missing.
    MetricTimestampNotFound { name: Arc<str> },

    /// A [`TryFromIntError`] encountered while converting integer values from [`i32`] to
    /// [`usize`].
    IntConversion(TryFromIntError),
}

impl Display for MetricParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let parse_error = "metric parse error:";

        match self {
            MetricParseError::Io(error) => write!(f, "{parse_error} I/O error: {error}"),
            MetricParseError::BsonDeserialzation(error) => write!(f, "{parse_error} BSON deserialization error: {error}"),
            MetricParseError::FieldAccess(error) => write!(f, "{parse_error} could not read the document field: {error}"),
            MetricParseError::MetricsCountMismatch => write!(f,
                "{parse_error} metrics count from the reference document and metrics count from samples do not match"
            ),
            MetricParseError::MetricTimestampNotFound { name } => write!(f, "{parse_error} the metric timestamps for the \"{}\" metric could not be found", name),
            MetricParseError::IntConversion(error) => write!(f, "{parse_error} could not parse integer: {error}"),
            MetricParseError::UnknownDocumentKind(value) => write!(f, "{parse_error} unknonw document type: {value}"),
        }
    }
}

impl Error for MetricParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MetricParseError::Io(error) => Some(error),
            MetricParseError::BsonDeserialzation(error) => Some(error),
            MetricParseError::FieldAccess(error) => Some(error),
            MetricParseError::MetricsCountMismatch => None,
            MetricParseError::MetricTimestampNotFound { .. } => None,
            MetricParseError::IntConversion(error) => Some(error),
            MetricParseError::UnknownDocumentKind(_) => None,
        }
    }
}

impl From<io::Error> for MetricParseError {
    fn from(error: io::Error) -> Self {
        MetricParseError::Io(Arc::new(error))
    }
}

impl From<de::Error> for MetricParseError {
    fn from(error: de::Error) -> Self {
        MetricParseError::BsonDeserialzation(error)
    }
}

impl From<TryFromIntError> for MetricParseError {
    fn from(error: TryFromIntError) -> Self {
        MetricParseError::IntConversion(error)
    }
}

impl From<KeyAccessError> for MetricParseError {
    fn from(error: KeyAccessError) -> Self {
        MetricParseError::FieldAccess(error)
    }
}

/// The error type for accessing BSON fields with the specified key.
#[derive(Debug, Clone)]
pub enum KeyAccessError {
    /// Could not find the field with the specified key.
    KeyNotFound { key: String },

    /// The field with the specified key was found, but not with the expected type.
    UnexpectedKeyType { key: String },

    /// Could not access field value with the specified key.
    AccessError { key: String },
}

impl Display for KeyAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let key_access_error = "key access error:";
        match *self {
            KeyAccessError::KeyNotFound { ref key } => {
                write!(f, "{key_access_error} could not find the field with the \"{}\" key", key)
            }
            KeyAccessError::UnexpectedKeyType { ref key } => write!(
                f,
                "{key_access_error} the field with \"{}\" key was found, but not with the expected type",
                key
            ),
            KeyAccessError::AccessError { ref key } => {
                write!(
                    f,
                    "{key_access_error} could not access the field value with the \"{}\" key",
                    key
                )
            }
        }
    }
}

impl Error for KeyAccessError {}

pub(crate) trait ValueAccessResultExt<T> {
    fn map_value_access_err(self, key: &str) -> Result<T, KeyAccessError>;
}

impl<T> ValueAccessResultExt<T> for Result<T, ValueAccessError> {
    fn map_value_access_err(self, key: &str) -> Result<T, KeyAccessError> {
        self.map_err(|error| match error {
            ValueAccessError::NotPresent => KeyAccessError::KeyNotFound {
                key: key.to_owned(),
            },
            ValueAccessError::UnexpectedType => KeyAccessError::UnexpectedKeyType {
                key: key.to_owned(),
            },
            _ => KeyAccessError::AccessError {
                key: key.to_owned(),
            },
        })
    }
}
