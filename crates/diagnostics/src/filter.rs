use bson::Document;
use chrono::DateTime;
use chrono::Utc;

use crate::error::MetricsDecoderError;
use crate::error::ValueAccessResultExt;

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const DATA_TYPE_KEY: &str = "type";
const ID_KEY: &str = "_id";

#[derive(Debug, Default)]
pub struct MetricsFilter {
    pub(crate) hostname: Option<String>,
    pub(crate) start_timestamp: Option<DateTime<Utc>>,
    pub(crate) end_timestamp: Option<DateTime<Utc>>,
}

impl MetricsFilter {
    pub fn by_timestamp(&self, timestamp: DateTime<Utc>) -> bool {
        match (self.start_timestamp, self.end_timestamp) {
            (None, None) => true,
            (None, Some(ref end)) => timestamp.le(end),
            (Some(ref start), None) => timestamp.ge(start),
            (Some(ref start), Some(ref end)) => timestamp.ge(start) && timestamp.le(end),
        }
    }

    pub fn by_hostname(&self, hostname: &str) -> bool {
        match self.hostname {
            Some(ref host) => host == hostname,
            None => true,
        }
    }
}

pub(crate) fn metrics_chunk(document: &Document) -> Result<bool, MetricsDecoderError> {
    document
        .get_i32(DATA_TYPE_KEY)
        .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricsDecoderError::from)
}

pub(crate) fn timestamp(
    document: &Document,
    filter: &MetricsFilter,
) -> Result<bool, MetricsDecoderError> {
    document
        .get_datetime(ID_KEY)
        .map(|ts| filter.by_timestamp(ts.to_chrono()))
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricsDecoderError::from)
}
