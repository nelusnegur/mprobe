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
use chrono::Duration;
use chrono::Utc;

use crate::bson::DocumentKind;
use crate::bson::ReadDocument;
use crate::error::MetricsDecoderError;
use crate::filter::HostnameFilter;
use crate::filter::TimeWindow;
use crate::iter::IteratorExt;
use crate::metrics::MetricsChunk;
use crate::MetricsFilter;

/// An iterator that reads recursively diagnostic data files from a root directory
/// identified by a [`std::fs::Path`], decodes metrics from BSON documents
/// and yields [`MetricsChunk`] elements.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MetricsIterator {
    metric_chunks: Box<dyn Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>>,
}

impl MetricsIterator {
    pub(crate) fn new(root_dir: ReadDir, filter: MetricsFilter) -> Self {
        let time_window = Rc::new(TimeWindow::new(
            filter.start_timestamp,
            filter.end_timestamp,
        ));

        let traverse_dir = TraverseDir::new(root_dir);
        let path_sorter = PathSorter::new(traverse_dir);
        let path_filter = PathFilter::new(path_sorter, time_window.clone());

        let file_reader = FileReader::new(path_filter);
        let hostname_filter = HostnameFilter::new(file_reader, filter.hostname);
        let timestamp_filter = hostname_filter
            // TODO: Filtering only by _id timestamp may miss documents
            .try_filter(move |d| d.timestamp().map(|ts| time_window.includes(&ts)));

        let metrics_chunk_filter =
            timestamp_filter.try_filter(|d| d.kind().map(|dt| dt == DocumentKind::MetricsChunk));
        let metrics_reader = MetricsChunkReader::new(metrics_chunk_filter);
        let metric_chunks = Box::new(metrics_reader);

        Self { metric_chunks }
    }
}

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

#[derive(Debug)]
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
/// filtering paths based on the file name.
/// It assumes the items in the inner iterator are yielded sorted
/// in ascending order.
#[must_use = "iterators are lazy and do nothing unless consumed"]
struct PathFilter<I> {
    iter: I,
    time_window: Rc<TimeWindow>,
    time_margin: Duration,
}

impl<I> PathFilter<I>
where
    I: Iterator<Item = Result<FileInfo, io::Error>>,
{
    fn new(iter: I, time_window: Rc<TimeWindow>) -> Self {
        Self {
            iter,
            time_window,
            time_margin: Duration::hours(4),
        }
    }
}

impl<I> Iterator for PathFilter<I>
where
    I: Iterator<Item = Result<FileInfo, io::Error>>,
{
    type Item = Result<FileInfo, io::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Ok(fi) => {
                    if self
                        .time_window
                        .includes_with_margin(&fi.timestamp, self.time_margin)
                    {
                        return Some(Ok(fi));
                    }
                }
                item => return Some(item),
            }
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
                None => {
                    if let Some(iter) = self.iter.take() {
                        let mut vec = Vec::from_iter(iter);
                        vec.sort_by_cached_key(|key| match key {
                            Ok(fi) => (fi.timestamp, fi.uid),
                            Err(_) => (Utc::now(), 0),
                        });

                        self.paths = Some(Box::new(vec.into_iter()));
                        continue;
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug)]
struct FileReader<I> {
    iter: I,
    inner_iter: Option<BsonReader<File>>,
}

impl<I> FileReader<I> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            inner_iter: None,
        }
    }
}

impl<I> Iterator for FileReader<I>
where
    I: Iterator<Item = Result<FileInfo, io::Error>>,
{
    type Item = Result<Document, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner_iter {
                Some(ref mut inner_iter) => match inner_iter.next() {
                    None => self.inner_iter = None,
                    item => return item.map(|i| i.map_err(MetricsDecoderError::from)),
                },
                None => match self.iter.next()? {
                    Ok(fi) => match File::open(fi.path) {
                        Ok(file) => self.inner_iter = Some(BsonReader::new(file)),
                        Err(err) => return Some(Err(MetricsDecoderError::from(err))),
                    },
                    Err(err) => return Some(Err(MetricsDecoderError::from(err))),
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

/// An iterator that yields BSON documents fron an underlying [`Read`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
struct MetricsChunkReader<I> {
    iter: I,
}

impl<I> MetricsChunkReader<I>
where
    I: Iterator<Item = Result<Document, MetricsDecoderError>>,
{
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> Iterator for MetricsChunkReader<I>
where
    I: Iterator<Item = Result<Document, MetricsDecoderError>>,
{
    type Item = Result<MetricsChunk, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| {
            item.and_then(|document| {
                document.metrics_chunk().and_then(|data| {
                    let mut data = Cursor::new(data);
                    MetricsChunk::from_reader(&mut data)
                })
            })
        })
    }
}
