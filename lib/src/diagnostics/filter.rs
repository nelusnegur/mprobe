use std::fs::DirEntry;
use std::io;
use std::path::Path;

use bson::Document;

use super::error::KeyValueAccessError;
use super::error::MetricParserError;
use super::error::ValueAccessResultExt;

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const DATA_TYPE_KEY: &str = "type";

pub(in crate::diagnostics) trait MetricsFilter {
    fn filter(document: &Document) -> Result<bool, MetricParserError>;
}

#[derive(Debug)]
pub(in crate::diagnostics) struct DefaultMetricsFilter;

impl MetricsFilter for DefaultMetricsFilter {
    fn filter(document: &Document) -> Result<bool, MetricParserError> {
        document
            .get_i32(DATA_TYPE_KEY)
            .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
            .map_value_access_err(DATA_TYPE_KEY)
            .map_err(MetricParserError::from)
    }
}

pub(crate) fn metrics_chunk(document: &Document) -> Result<bool, MetricParserError> {
    document
        .get_i32(DATA_TYPE_KEY)
        .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricParserError::from)
}

pub(crate) fn file_predicate(entry: &io::Result<DirEntry>) -> bool {
    entry
        .as_ref()
        .map_or(true, |e| is_not_interim_file(&e.path()))
}

fn is_not_interim_file(path: &Path) -> bool {
    path.extension()
        .map_or(true, |extension| extension != "interim")
}
