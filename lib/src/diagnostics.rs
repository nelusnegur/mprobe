mod compression;
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
    dir_entries: ReadDir,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> io::Result<Self> {
        let dir_entries = fs::read_dir(path)?;

        Ok(DiagnosticData { path, dir_entries })
    }

    pub fn scan(&self) -> io::Result<()> {
        println!("Scanning diagnostic data from {:?}", self.path);

        for entry in fs::read_dir(self.path)? {
            let dir = entry?;
            println!("{:?}", dir.path());

            DiagnosticData::scan_dir(&dir.path())?;
        }

        Ok(())
    }

    fn scan_dir(path: &'a Path) -> io::Result<()> {
        println!("Scanning directory {path:?}");

        for entry in fs::read_dir(path)? {
            let dir = entry?;
            let dir_path = &dir.path();

            println!("{:?}", dir_path);

            if let Some(extension) = dir_path.extension() {
                if extension != "interim" {
                    DiagnosticData::scan_file(&dir.path())?;
                }
            }
        }

        Ok(())
    }

    fn scan_file(path: &'a Path) -> io::Result<()> {
        println!("Scanning file {path:?}");

        let file = fs::File::open(path)?;
        let mut buf_reader = BufReader::new(file);

        let mut metric_docs: Vec<Document> = Vec::new();

        while let Ok(doc) = Document::from_reader(&mut buf_reader) {
            if let Ok(doc_type) = doc.get_i32("type") {
                if doc_type == 1 {
                    metric_docs.push(doc);
                }
            }
        }

        DiagnosticData::scan_metrics(&metric_docs)?;

        Ok(())
    }

    fn scan_metrics(metric_docs: &Vec<Document>) -> io::Result<Vec<MetricsChunk>> {
        let mut chunks: Vec<MetricsChunk> = Vec::new();

        for metric_doc in metric_docs {
            let mut data = match metric_doc.get_binary_generic("data") {
                Ok(it) => Cursor::new(it),
                Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err)),
            };

            let x = MetricsChunk::from_reader(&mut data)?;
            println!(
                "{:?}",
                x.metrics
                    .iter()
                    .find(|x| x.name.ends_with("wiredTiger/block-manager/blocks read"))
            );

            chunks.push(x);
        }

        Ok(chunks)
    }
}

impl<'a> IntoIterator for DiagnosticData<'a> {
    type Item = io::Result<MetricsChunk>;

    type IntoIter = DiagnsticDataIter;

    fn into_iter(self) -> Self::IntoIter {
        DiagnsticDataIter::new(self.dir_entries)
    }
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum DocumentType {
    Metadata = 0,
    MetricsChunk = 1,
}

const METRICS_CHUNK_FIELD_NAME: &str = "data";
const DOCUMENT_TYPE_FIELD_NAME: &str = "type";

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
                    if let Ok(doc_type) = document.get_i32(DOCUMENT_TYPE_FIELD_NAME) {
                        if doc_type == DocumentType::MetricsChunk as i32 {
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
