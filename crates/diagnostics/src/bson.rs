use bson::Document;
use chrono::DateTime;
use chrono::Utc;

use crate::error::MetricParseError;
use crate::error::ValueAccessResultExt;

const ID_KEY: &str = "_id";
const DATA_TYPE_KEY: &str = "type";
const METADATA_KEY: &str = "doc";
const METRICS_CHUNK_KEY: &str = "data";

const COMMON_KEY: &str = "common";
const HOST_INFO_KEY: &str = "hostInfo";
const SYSTEM_KEY: &str = "system";
const HOSTNAME_KEY: &str = "hostname";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub(crate) enum DocumentKind {
    Metadata = 0,
    MetricsChunk,
    PeriodicMetadata,
}

impl TryFrom<i32> for DocumentKind {
    type Error = MetricParseError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DocumentKind::Metadata),
            1 => Ok(DocumentKind::MetricsChunk),
            2 => Ok(DocumentKind::PeriodicMetadata),
            val => Err(MetricParseError::UnknownDocumentKind(val)),
        }
    }
}

pub(crate) trait ReadDocument {
    fn kind(&self) -> Result<DocumentKind, MetricParseError>;
    fn timestamp(&self) -> Result<DateTime<Utc>, MetricParseError>;
    fn hostname(&self) -> Result<&str, MetricParseError>;
    fn metrics_chunk(&self) -> Result<&Vec<u8>, MetricParseError>;
}

impl ReadDocument for Document {
    fn kind(&self) -> Result<DocumentKind, MetricParseError> {
        self.get_i32(DATA_TYPE_KEY)
            .map_value_access_err(DATA_TYPE_KEY)
            .map_err(MetricParseError::from)
            .and_then(|dt| dt.try_into())
    }

    fn timestamp(&self) -> Result<DateTime<Utc>, MetricParseError> {
        self.get_datetime(ID_KEY)
            .map_value_access_err(ID_KEY)
            .map(|ts| ts.to_chrono())
            .map_err(MetricParseError::from)
    }

    fn hostname(&self) -> Result<&str, MetricParseError> {
        let metadata = self
            .get_document(METADATA_KEY)
            .map_value_access_err(METADATA_KEY)?;

        // In MongoDB 8.0 a new nested field, common, was introduced,
        // and we have to account for it as well until all the previous
        // versions are no longer supported.
        let common = match metadata.get_document(COMMON_KEY) {
            Ok(common) => common,
            Err(_) => metadata,
        };

        let host_info = common
            .get_document(HOST_INFO_KEY)
            .map_value_access_err(HOST_INFO_KEY)?;

        let system = host_info
            .get_document(SYSTEM_KEY)
            .map_value_access_err(SYSTEM_KEY)?;

        let hostname = system
            .get_str(HOSTNAME_KEY)
            .map_value_access_err(HOSTNAME_KEY)?;

        Ok(hostname)
    }

    fn metrics_chunk(&self) -> Result<&Vec<u8>, MetricParseError> {
        let data = self
            .get_binary_generic(METRICS_CHUNK_KEY)
            .map_value_access_err(METRICS_CHUNK_KEY)?;

        Ok(data)
    }
}
