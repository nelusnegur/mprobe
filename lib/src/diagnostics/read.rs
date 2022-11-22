use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::fs::ReadDir;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;

use bson::de;
use bson::Document;

use crate::diagnostics::error::MetricsDecoderError;
use crate::diagnostics::error::ValueAccessResultExt;
use crate::diagnostics::filter;
use crate::diagnostics::metrics::MetricsChunk;
use crate::iter::IteratorExt;

type FileReader = MetricsDecoder<
    BsonReader<BufReader<File>>,
    for<'d> fn(&'d Document) -> Result<bool, MetricsDecoderError>,
>;

/// An iterator that decodes metrics from a [`std::fs::DirEntry`] and
/// yields [`MetricsChunk`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct MetricsIterator {
    dirs: ReadDir,
    files: Option<ReadDir>,
    file_reader: Option<FileReader>,
}

impl MetricsIterator {
    pub fn new(dirs: ReadDir) -> Self {
        Self {
            dirs,
            files: None,
            file_reader: None,
        }
    }
}

impl Iterator for MetricsIterator {
    type Item = Result<MetricsChunk, MetricsDecoderError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(reader) = &mut self.file_reader {
                match reader.next() {
                    Some(metrics_chunk) => return Some(metrics_chunk),
                    None => {
                        self.file_reader = None;
                        continue;
                    }
                }
            }

            if let Some(files) = &mut self.files {
                match files.find(filter::file_predicate) {
                    Some(Ok(entry)) => {
                        match File::open(entry.path()).map_err(MetricsDecoderError::from) {
                            Ok(file) => {
                                let buf_reader = BufReader::new(file);
                                let file_reader = BsonReader::new(buf_reader);
                                let metrics_decoder = MetricsDecoder::new(
                                    file_reader,
                                    filter::metrics_chunk
                                        as for<'d> fn(
                                            &Document,
                                        )
                                            -> Result<bool, MetricsDecoderError>,
                                );

                                self.file_reader = Some(metrics_decoder);
                                continue;
                            }
                            Err(err) => return Some(Err(err)),
                        }
                    }
                    Some(Err(err)) => return Some(Err(MetricsDecoderError::from(err))),
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
                .map_err(MetricsDecoderError::from)
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

const METRICS_CHUNK_KEY: &str = "data";

/// An iterator that filters elements of `iter` with `predicate`, decodes
/// metrics from BSON documents and yields [`MetricsChunk`].
#[must_use = "iterators are lazy and do nothing unless consumed"]
#[derive(Debug, Clone)]
struct MetricsDecoder<I, P> {
    iter: I,
    predicate: P,
}

impl<I, P> MetricsDecoder<I, P> {
    fn new(iter: I, predicate: P) -> MetricsDecoder<I, P> {
        MetricsDecoder { iter, predicate }
    }

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

impl<I, P> Iterator for MetricsDecoder<I, P>
where
    I: Iterator<Item = Result<Document, MetricsDecoderError>>,
    P: FnMut(&Document) -> Result<bool, MetricsDecoderError>,
{
    type Item = Result<MetricsChunk, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        (&mut self.iter)
            .try_filter(&mut self.predicate)
            .map(MetricsDecoder::<I, P>::decode_metrics_chunk)
            .next()
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
    type Item = Result<Document, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match Document::from_reader(&mut self.reader) {
            Ok(document) => Some(Ok(document)),
            Err(de::Error::Io(err)) if err.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(error) => Some(Err(MetricsDecoderError::from(error))),
        }
    }
}
