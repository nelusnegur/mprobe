use std::fs;
use std::fs::File;
use std::fs::ReadDir;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;

use bson::de;
use bson::Document;

use crate::error::MetricsDecoderError;
use crate::error::ValueAccessResultExt;
use crate::filter;
use crate::iter::IteratorExt;
use crate::metrics::MetricsChunk;

/// An iterator that reads recursively diagnostic data files from a root directory
/// identified by a [`std::fs::Path`], decodes metrics from BSON documents
/// and yields [`MetricsChunk`] elements.
#[must_use = "iterators are lazy and do nothing unless consumed"]
// #[derive(Debug)]
pub struct MetricsIterator {
    metric_chunks: Box<dyn Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>>,
}

impl MetricsIterator {
    pub(crate) fn new(root_dir: ReadDir) -> Self {
        let traverse_dir = TraverseDir::new(root_dir);
        let metrics_reader = Self::read_metrics(traverse_dir);
        let metric_chunks = Box::new(metrics_reader);

        Self { metric_chunks }
    }

    fn read_metrics<I>(iter: I) -> impl Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>
    where
        I: Iterator<Item = Result<PathBuf, io::Error>>,
    {
        iter.map(|item| {
            item.and_then(File::open)
                .map(|file| Self::decode_metrics(BsonReader::new(file)))
                .map_err(MetricsDecoderError::from)
        })
        .try_flatten()
    }

    fn decode_metrics<I>(iter: I) -> impl Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>
    where
        I: Iterator<Item = Result<Document, de::Error>>,
    {
        iter.map(|item| item.map_err(MetricsDecoderError::from))
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
    type Item = Result<PathBuf, io::Error>;

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
                            return Some(Ok(entry.path()));
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
