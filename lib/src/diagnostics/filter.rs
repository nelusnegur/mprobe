use std::fs::DirEntry;
use std::io;
use std::path::Path;

use bson::Document;

use crate::diagnostics::error::MetricsDecoderError;
use crate::diagnostics::error::ValueAccessResultExt;

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const DATA_TYPE_KEY: &str = "type";

pub(crate) fn metrics_chunk(document: &Document) -> Result<bool, MetricsDecoderError> {
    document
        .get_i32(DATA_TYPE_KEY)
        .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricsDecoderError::from)
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
