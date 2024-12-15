use std::fs;
use std::fs::File;
use std::fs::ReadDir;
use std::io;
use std::io::Cursor;
use std::io::ErrorKind;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use bson::de;
use bson::Document;
use chrono::DateTime;
use chrono::Utc;

use crate::error::MetricsDecoderError;
use crate::error::ValueAccessResultExt;
use crate::filter;
use crate::filter::HostnameFilter;
use crate::filter::MetricsFilter;
use crate::iter::IteratorExt;
use crate::metrics::MetricsChunk;

/// An iterator that reads recursively diagnostic data files from a root directory
/// identified by a [`std::fs::Path`], decodes metrics from BSON documents
/// and yields [`MetricsChunk`] elements.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MetricsIterator {
    metric_chunks: Box<dyn Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>>,
}

impl MetricsIterator {
    pub(crate) fn new(root_dir: ReadDir, filter: MetricsFilter) -> Self {
        let filter = Rc::new(filter);
        let traverse_dir = TraverseDir::new(root_dir);
        let path_sorter = PathSorter::new(traverse_dir);
        // let path_filter = Self::filter_path(path_sorter, filter.clone());

        let metrics_reader = Self::read_metrics(path_sorter, filter.clone());
        let metric_chunks = Box::new(metrics_reader);

        Self { metric_chunks }
    }

    fn filter_path<I>(
        iter: I,
        filter: Rc<MetricsFilter>,
    ) -> impl Iterator<Item = Result<FileInfo, io::Error>>
    where
        I: Iterator<Item = Result<FileInfo, io::Error>>,
    {
        iter.filter(move |item| match item {
            Ok(fi) => filter.by_timestamp(fi.timestamp),
            Err(_) => true,
        })
    }

    fn read_metrics<I>(
        iter: I,
        filter: Rc<MetricsFilter>,
    ) -> impl Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>
    where
        I: Iterator<Item = Result<FileInfo, io::Error>>,
    {
        iter.map(move |item| {
            item.and_then(|f| File::open(f.path))
                .map(|file| Self::decode_metrics(BsonReader::new(file), filter.clone()))
                .map_err(MetricsDecoderError::from)
        })
        .try_flatten()
    }

    fn decode_metrics<I>(
        iter: I,
        filter: Rc<MetricsFilter>,
    ) -> impl Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>
    where
        I: Iterator<Item = Result<Document, de::Error>>,
    {
        let bson_documents = iter.map(|item| item.map_err(MetricsDecoderError::from));
        let hostname_filter = HostnameFilter::new(bson_documents, filter.hostname.clone());

        hostname_filter
            // TODO: Filtering only by _id timestamp may miss documents
            .try_filter(move |d| filter::timestamp(d, &filter))
            .try_filter(filter::metrics_chunk)
            .map(Self::decode_metrics_chunk)
    }

    #[inline]
    fn decode_metrics_chunk(
        item: Result<Document, MetricsDecoderError>,
    ) -> Result<MetricsChunk, MetricsDecoderError> {
        match item {
            Ok(document) => {
                let data = document
                    .get_binary_generic(METRICS_CHUNK_KEY)
                    .map_value_access_err(METRICS_CHUNK_KEY)?;

                let mut data = Cursor::new(data);
                MetricsChunk::from_reader(&mut data)
            }
            Err(error) => Err(error),
        }
    }
}

const METRICS_CHUNK_KEY: &str = "data";

impl Iterator for MetricsIterator {
    type Item = Result<MetricsChunk, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.metric_chunks.next()
    }
}

/// An iterator that traverses recursively a directory tree identified by
/// a [`std::fs::Path`] and yields [`std::path::PathBuf`] for the contained files only.
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
struct TraverseDir {
    dirs: Vec<ReadDir>,
}

impl TraverseDir {
    fn new(root_dir: ReadDir) -> Self {
        let dirs = vec![root_dir];
        Self { dirs }
    }
}

impl Iterator for TraverseDir {
    type Item = Result<FileInfo, io::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.last_mut()?;

            match dir.next() {
                Some(Ok(entry)) => match entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            match fs::read_dir(entry.path()) {
                                Ok(next_dir) => self.dirs.push(next_dir),
                                Err(error) => return Some(Err(error)),
                            }
                        } else if file_type.is_file() {
                            // TODO: Process the .interim file last
                            // For now just skip it.
                            let path = entry.path();
                            if path.extension().is_none_or(|e| e == "interim") {
                                continue;
                            }

                            let file_info = FileInfo::from(path);
                            return Some(file_info);
                        } else {
                            continue;
                        }
                    }
                    Err(error) => return Some(Err(error)),
                },
                Some(Err(error)) => return Some(Err(error)),
                None => {
                    self.dirs.pop();
                }
            }
        }
    }
}

struct FileInfo {
    path: PathBuf,
    timestamp: DateTime<Utc>,
    uid: u16,
}

impl FileInfo {
    pub fn from(path: PathBuf) -> Result<FileInfo, io::Error> {
        let (timestamp, uid) = match path.extension() {
            Some(extension) => {
                let extension = extension.to_str().ok_or_else(|| {
                    io::Error::new(
                        ErrorKind::InvalidData,
                        format!("the file extension ({extension:?}) is not valid UTF-8"),
                    )
                })?;

                Self::parse_timestamp_and_uid(extension)
            }
            None => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("the '{path:?}' path does not have a file extension"),
                ))
            }
        }?;

        Ok(Self {
            path,
            timestamp,
            uid,
        })
    }

    fn parse_timestamp_and_uid(extension: &str) -> Result<(DateTime<Utc>, u16), io::Error> {
        const TIMESTAMP_FORMAT: &str = "%Y-%m-%dT%H-%M-%S%#z";

        match extension.rsplit_once("-") {
            Some((ts, uid)) => {
                let ts = DateTime::parse_from_str(ts, TIMESTAMP_FORMAT)
                    .map_err(|e| {
                        io::Error::new(
                            ErrorKind::Other,
                            format!("parsing file extension timestamp ({ts}) failed: {e}"),
                        )
                    })?
                    .with_timezone(&Utc);

                let uid = uid.parse::<u16>().map_err(|e| {
                    io::Error::new(
                        ErrorKind::Other,
                        format!("parsing file extension uid ({uid}) failed: {e}"),
                    )
                })?;

                Ok((ts, uid))
            }
            None => Err(io::Error::new(
                ErrorKind::Other,
                format!("splitting file extension ({extension}) into ts and uid failed"),
            )),
        }
    }
}

/// An iterator that traverses the given [`std::path::PathBuf`]s
/// yielding the paths in sorterd order.
#[must_use = "iterators are lazy and do nothing unless consumed"]
struct PathSorter<I> {
    iter: Option<I>,
    paths: Option<Box<dyn Iterator<Item = Result<FileInfo, io::Error>>>>,
}

impl<I> PathSorter<I>
where
    I: Iterator<Item = Result<FileInfo, io::Error>>,
{
    fn new(iter: I) -> Self {
        Self {
            iter: Some(iter),
            paths: None,
        }
    }
}

impl<I> Iterator for PathSorter<I>
where
    I: Iterator<Item = Result<FileInfo, io::Error>>,
{
    type Item = Result<FileInfo, io::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.paths.as_mut() {
                Some(paths) => return paths.next(),
                None => match self.iter.take() {
                    Some(iter) => {
                        let mut vec = Vec::from_iter(iter);
                        vec.sort_by_cached_key(|key| match key {
                            Ok(fi) => (fi.timestamp, fi.uid),
                            Err(_) => (Utc::now(), 0),
                        });

                        self.paths = Some(Box::new(vec.into_iter()));
                        continue;
                    }
                    None => return None,
                },
            }
        }
    }
}

/// An iterator that yields BSON documents fron an underlying [`Read`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
struct BsonReader<R> {
    reader: R,
}

impl<R> BsonReader<R> {
    fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> Iterator for BsonReader<R> {
    type Item = Result<Document, de::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match Document::from_reader(&mut self.reader) {
            Ok(document) => Some(Ok(document)),
            Err(de::Error::Io(error)) if error.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(error) => Some(Err(error)),
        }
    }
}
