mod archive;
mod directory;

use std::fs::ReadDir;
use std::io;
use std::io::Cursor;
use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;

use bson::Document;
use bson::de;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::MetricsFilter;
use crate::bson::DocumentKind;
use crate::bson::ReadDocument;
use crate::error::MetricParseError;
use crate::filter::HostnameFilter;
use crate::filter::TimeWindow;
use crate::filter::TimeWindowFilter;
use crate::iter::IteratorExt;
use crate::metrics::MetricsChunk;
use crate::read::directory::ReadDirectory;

/// An iterator that reads recursively diagnostic data files from a root directory
/// identified by a [`std::fs::Path`], decodes metrics from BSON documents
/// and yields [`MetricsChunk`] elements.
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MetricsIterator {
    metric_chunks: Box<dyn Iterator<Item = Result<MetricsChunk, MetricParseError>>>,
}

impl MetricsIterator {
    pub(crate) fn new(root_dir: ReadDir, filter: MetricsFilter) -> Self {
        let time_window = Rc::new(TimeWindow::new(filter.start, filter.end));

        let traverse_dir = ReadDirectory::new(root_dir);
        let path_sorter = PathSorter::new(traverse_dir);
        let path_filter = PathFilter::new(path_sorter, time_window.clone());

        let file_reader = FileReader::new(path_filter);
        let hostname_filter = HostnameFilter::new(file_reader, filter.hostname);
        let time_window_filter = TimeWindowFilter::new(hostname_filter, time_window.clone());

        let metrics_chunk_filter =
            time_window_filter.try_filter(|d| d.kind().map(|k| k == DocumentKind::MetricsChunk));
        let metrics_reader = MetricsChunkReader::new(metrics_chunk_filter);
        let chunk_filter = metrics_reader
            .try_filter(move |chunk| Ok(time_window.overlaps(&chunk.start, &chunk.end)));
        let metric_chunks = Box::new(chunk_filter);

        Self { metric_chunks }
    }
}

impl Iterator for MetricsIterator {
    type Item = Result<MetricsChunk, MetricParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.metric_chunks.next()
    }
}

#[derive(Debug)]
struct ReadItem<R: Read> {
    reader: R,
    timestamp: DateTime<Utc>,
    uid: u16,
}

impl<R: Read> ReadItem<R> {
    pub fn new(path: &Path, reader: R) -> Result<ReadItem<R>, io::Error> {
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
                ));
            }
        }?;

        Ok(Self {
            reader,
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
                        io::Error::other(format!(
                            "parsing file extension timestamp ({ts}) failed: {e}"
                        ))
                    })?
                    .with_timezone(&Utc);

                let uid = uid.parse::<u16>().map_err(|e| {
                    io::Error::other(format!("parsing file extension uid ({uid}) failed: {e}"))
                })?;

                Ok((ts, uid))
            }
            None => Err(io::Error::other(format!(
                "splitting file extension ({extension}) into ts and uid failed"
            ))),
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

impl<I, R> PathFilter<I>
where
    I: Iterator<Item = Result<ReadItem<R>, io::Error>>,
    R: Read,
{
    fn new(iter: I, time_window: Rc<TimeWindow>) -> Self {
        Self {
            iter,
            time_window,
            time_margin: Duration::hours(4),
        }
    }
}

impl<I, R: Read> Iterator for PathFilter<I>
where
    I: Iterator<Item = Result<ReadItem<R>, io::Error>>,
{
    type Item = Result<ReadItem<R>, io::Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Ok(ri) => {
                    if self
                        .time_window
                        .includes_with_margin(&ri.timestamp, self.time_margin)
                    {
                        return Some(Ok(ri));
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
struct PathSorter<I, R: Read> {
    iter: Option<I>,
    paths: Option<Box<dyn Iterator<Item = Result<ReadItem<R>, io::Error>>>>,
}

impl<I, R> PathSorter<I, R>
where
    I: Iterator<Item = Result<ReadItem<R>, io::Error>>,
    R: Read + 'static,
{
    fn new(iter: I) -> Self {
        Self {
            iter: Some(iter),
            paths: None,
        }
    }
}

impl<I, R> Iterator for PathSorter<I, R>
where
    I: Iterator<Item = Result<ReadItem<R>, io::Error>>,
    R: Read + 'static,
{
    type Item = Result<ReadItem<R>, io::Error>;

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
struct FileReader<I, R: Read> {
    iter: I,
    inner_iter: Option<BsonReader<R>>,
}

impl<I, R: Read> FileReader<I, R> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            inner_iter: None,
        }
    }
}

impl<I, R: Read> Iterator for FileReader<I, R>
where
    I: Iterator<Item = Result<ReadItem<R>, io::Error>>,
{
    type Item = Result<Document, MetricParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner_iter {
                Some(ref mut inner_iter) => match inner_iter.next() {
                    None => self.inner_iter = None,
                    item => return item.map(|i| i.map_err(MetricParseError::from)),
                },
                None => match self.iter.next()? {
                    Ok(ri) => self.inner_iter = Some(BsonReader::new(ri.reader)),
                    Err(err) => return Some(Err(MetricParseError::from(err))),
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

impl<R: Read> BsonReader<R> {
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
    I: Iterator<Item = Result<Document, MetricParseError>>,
{
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I: Iterator> Iterator for MetricsChunkReader<I>
where
    I: Iterator<Item = Result<Document, MetricParseError>>,
{
    type Item = Result<MetricsChunk, MetricParseError>;

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
