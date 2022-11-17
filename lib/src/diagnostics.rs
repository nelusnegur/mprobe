mod compression;
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

use crate::diagnostics::metrics::MetricsChunk;

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub path: &'a Path,
    entries: ReadDir,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> io::Result<Self> {
        let entries = fs::read_dir(path)?;

        Ok(DiagnosticData { path, entries })
    }
}

impl<'a> IntoIterator for DiagnosticData<'a> {
    type Item = io::Result<MetricsChunk>;

    type IntoIter = DiagnsticDataIter;

    fn into_iter(self) -> Self::IntoIter {
        DiagnsticDataIter::new(self.entries)
    }
}

const METRICS_CHUNK_DATA_TYPE: i32 = 1;
const METRICS_CHUNK_FIELD_NAME: &str = "data";
const DATA_TYPE_FIELD_NAME: &str = "type";

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

    fn read_metrics_chunk<R: Read>(mut reader: R) -> Option<io::Result<MetricsChunk>> {
        loop {
            match Document::from_reader(&mut reader) {
                Ok(document) => {
                    if let Ok(data_type) = document.get_i32(DATA_TYPE_FIELD_NAME) {
                        if data_type == METRICS_CHUNK_DATA_TYPE {
                            return match document.get_binary_generic(METRICS_CHUNK_FIELD_NAME) {
                                Ok(data) => {
                                    let mut data = Cursor::new(data);
                                    Some(MetricsChunk::from_reader(&mut data))
                                }
                                Err(err) => {
                                    Some(Err(io::Error::new(io::ErrorKind::InvalidData, err)))
                                }
                            };
                        }
                    }
                }
                Err(bson::de::Error::Io(err)) if err.kind() == io::ErrorKind::UnexpectedEof => {
                    return None
                }
                Err(err) => return Some(Err(io::Error::new(io::ErrorKind::InvalidData, err))),
            }
        }
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
    type Item = io::Result<MetricsChunk>;

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
                    Some(Ok(entry)) => match fs::File::open(entry.path()) {
                        Ok(file) => {
                            let buf_reader = BufReader::new(file);
                            self.file_reader = Some(buf_reader);
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    },
                    Some(Err(err)) => return Some(Err(err)),
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
