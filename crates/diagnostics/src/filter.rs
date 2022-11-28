use bson::Document;

use crate::error::MetricsDecoderError;
use crate::error::ValueAccessResultExt;

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const DATA_TYPE_KEY: &str = "type";

pub(crate) fn metrics_chunk(document: &Document) -> Result<bool, MetricsDecoderError> {
    document
        .get_i32(DATA_TYPE_KEY)
        .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricsDecoderError::from)
}
