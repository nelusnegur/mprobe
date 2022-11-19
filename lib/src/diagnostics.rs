mod compression;
mod error;
mod metadata;
mod metrics;

use bson::Document;
use std::fs;
use std::fs::DirEntry;
use std::fs::File;
use std::fs::ReadDir;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;

use crate::diagnostics::error::MetricParserError;
use crate::diagnostics::error::ValueAccessResultExt;
use crate::diagnostics::metrics::MetricsChunk;

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub path: &'a Path,
    entries: ReadDir,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;

        Ok(DiagnosticData { path, entries })
    }
}

impl<'a> IntoIterator for DiagnosticData<'a> {
    type Item = Result<MetricsChunk, MetricParserError>;

    type IntoIter = DiagnsticDataIter;

    fn into_iter(self) -> Self::IntoIter {
        DiagnsticDataIter::new(self.entries)
    }
}

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const METRICS_CHUNK_KEY: &str = "data";
const DATA_TYPE_KEY: &str = "type";

#[derive(Debug)]
pub struct DiagnsticDataIter {
    dirs: ReadDir,
    files: Option<ReadDir>,
    file_reader: Option<BufReader<File>>,
}

impl DiagnsticDataIter {
    pub fn new(dirs: ReadDir) -> Self {
        Self {
            dirs,
            files: None,
            file_reader: None,
        }
    }

    fn read_metrics_chunk<R: Read>(
        mut reader: R,
    ) -> Option<Result<MetricsChunk, MetricParserError>> {
        loop {
            match Document::from_reader(&mut reader).map_err(MetricParserError::from) {
                Ok(document) => {
                    if let Ok(data_type) = document.get_i32(DATA_TYPE_KEY) {
                        if data_type == METRICS_CHUNK_DATA_TYPE {
                            let chunk = DiagnsticDataIter::read_chunk(document);
                            return Some(chunk);
                        }
                    }
                }
                Err(MetricParserError::BsonDeserialzation(bson::de::Error::Io(err)))
                    if err.kind() == io::ErrorKind::UnexpectedEof =>
                {
                    return None
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }

    fn read_chunk(document: Document) -> Result<MetricsChunk, MetricParserError> {
        let data = document
            .get_binary_generic(METRICS_CHUNK_KEY)
            .map_value_access_err(METRICS_CHUNK_KEY)?;

        let mut data = Cursor::new(data);
        MetricsChunk::from_reader(&mut data)
    }

    fn file_predicate(entry: &io::Result<DirEntry>) -> bool {
        entry
            .as_ref()
            .map_or(true, |e| DiagnsticDataIter::is_not_interim_file(&e.path()))
    }

    fn is_not_interim_file(path: &Path) -> bool {
        path.extension()
            .map_or(true, |extension| extension != "interim")
    }
}

impl Iterator for DiagnsticDataIter {
    type Item = Result<MetricsChunk, MetricParserError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(reader) = &mut self.file_reader {
                match DiagnsticDataIter::read_metrics_chunk(reader) {
                    Some(metrics_chunk) => return Some(metrics_chunk),
                    None => {
                        self.file_reader = None;
                        continue;
                    }
                }
            }

            if let Some(files) = &mut self.files {
                match files.find(DiagnsticDataIter::file_predicate) {
                    Some(Ok(entry)) => {
                        match fs::File::open(entry.path()).map_err(MetricParserError::from) {
                            Ok(file) => {
                                let buf_reader = BufReader::new(file);
                                self.file_reader = Some(buf_reader);
                                continue;
                            }
                            Err(err) => return Some(Err(err)),
                        }
                    }
                    Some(Err(err)) => return Some(Err(MetricParserError::from(err))),
                    None => {
                        self.files = None;
                        continue;
                    }
                }
            }

            match self
                .dirs
                .next()?
                .and_then(|entry| fs::read_dir(entry.path()))
                .map_err(MetricParserError::from)
            {
                Ok(dir) => {
                    self.files = Some(dir);
                    continue;
                }
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
