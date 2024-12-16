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

/// The error type for decoding diagnostic metrics.
///
/// Errors mostly originate from I/O read operations, BSON deserialization and field value accesses.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MetricsDecoderError {
    /// A [`std::io::Error`] encountered while reading metric chunks.
    Io(Arc<io::Error>),

    /// A [`bson::de::Error`] encountered while deserializing BSON documents.
    BsonDeserialzation(de::Error),

    /// A [`KeyValueAccessError`] encountered while accessing BSON fields.
    KeyValueAccess(KeyValueAccessError),

    /// Unknown document type.
    UnknownDocumentKind(i32),

    /// The metrics count from the reference document and the metrics count from samples do not
    /// match.
    MetricsCountMismatch,

    /// The collector type could not be extracted from the metric name.
    MetricCollectorNotFound,

    /// The metric with the specified name could not be found.
    #[non_exhaustive]
    MetricNotFound { name: String },

    /// The metric with the specified name does not contain any value.
    #[non_exhaustive]
    MetricValueNotFound { name: String },

    /// A [`TryFromIntError`] encountered while converting integer values from [`i32`] to
    /// [`usize`].
    IntConversion(TryFromIntError),
}

impl Display for MetricsDecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MetricsDecoderError::Io(inner) => Display::fmt(inner, f),
            MetricsDecoderError::BsonDeserialzation(inner) => Display::fmt(inner, f),
            MetricsDecoderError::KeyValueAccess(inner) => Display::fmt(inner, f),
            MetricsDecoderError::MetricsCountMismatch => f.write_str(
                "metrics count from the reference document and metrics count from samples do not match"
            ),
            MetricsDecoderError::MetricCollectorNotFound => f.write_str(
                "collector type could not be extracted from the metric name"
            ),
            MetricsDecoderError::MetricNotFound { name } => write!(f, "\"{}\" metric not found", name),
            MetricsDecoderError::MetricValueNotFound { name } => write!(f, "there are no values for \"{}\" metric", name),
            MetricsDecoderError::IntConversion(inner) => Display::fmt(inner, f),
            MetricsDecoderError::UnknownDocumentKind(value) => write!(f, "Unknonw document type value: {value}"),
        }
    }
}

impl Error for MetricsDecoderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MetricsDecoderError::Io(inner) => Some(inner),
            MetricsDecoderError::BsonDeserialzation(inner) => Some(inner),
            MetricsDecoderError::KeyValueAccess(inner) => Some(inner),
            MetricsDecoderError::MetricsCountMismatch => None,
            MetricsDecoderError::MetricCollectorNotFound => None,
            MetricsDecoderError::MetricNotFound { .. } => None,
            MetricsDecoderError::MetricValueNotFound { .. } => None,
            MetricsDecoderError::IntConversion(inner) => Some(inner),
            MetricsDecoderError::UnknownDocumentKind(_) => None,
        }
    }
}

impl From<io::Error> for MetricsDecoderError {
    fn from(error: io::Error) -> Self {
        MetricsDecoderError::Io(Arc::new(error))
    }
}

impl From<de::Error> for MetricsDecoderError {
    fn from(error: de::Error) -> Self {
        MetricsDecoderError::BsonDeserialzation(error)
    }
}

impl From<TryFromIntError> for MetricsDecoderError {
    fn from(error: TryFromIntError) -> Self {
        MetricsDecoderError::IntConversion(error)
    }
}

impl From<KeyValueAccessError> for MetricsDecoderError {
    fn from(error: KeyValueAccessError) -> Self {
        MetricsDecoderError::KeyValueAccess(error)
    }
}

/// The error type for accessing BSON fields with the specified key.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum KeyValueAccessError {
    /// Could not find the field with the specified key.
    #[non_exhaustive]
    KeyNotFound { key: String },

    /// The field with the specified key was found, but not with the expected type.
    #[non_exhaustive]
    UnexpectedKeyType { key: String },

    /// Could not access field value with the specified key.
    #[non_exhaustive]
    AccessError { key: String },
}

impl Display for KeyValueAccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            KeyValueAccessError::KeyNotFound { ref key } => {
                write!(f, "could not find the field with the \"{}\" key", key)
            }
            KeyValueAccessError::UnexpectedKeyType { ref key } => write!(
                f,
                "the field with \"{}\" key was found, but not with the expected type",
                key
            ),
            KeyValueAccessError::AccessError { ref key } => {
                write!(
                    f,
                    "could not access the field value with the \"{}\" key",
                    key
                )
            }
        }
    }
}

impl Error for KeyValueAccessError {}

pub(crate) trait ValueAccessResultExt<T> {
    fn map_value_access_err(self, key: &str) -> Result<T, KeyValueAccessError>;
}

impl<T> ValueAccessResultExt<T> for Result<T, ValueAccessError> {
    fn map_value_access_err(self, key: &str) -> Result<T, KeyValueAccessError> {
        self.map_err(|error| match error {
            ValueAccessError::NotPresent => KeyValueAccessError::KeyNotFound {
                key: key.to_owned(),
            },
            ValueAccessError::UnexpectedType => KeyValueAccessError::UnexpectedKeyType {
                key: key.to_owned(),
            },
            _ => KeyValueAccessError::AccessError {
                key: key.to_owned(),
            },
        })
    }
}
