mod bytes;
mod compression;
mod filter;
mod iter;
mod read;

pub mod error;
pub mod metadata;
pub mod metrics;

use std::fs;
use std::fs::ReadDir;
use std::io;
use std::path::Path;

use crate::error::MetricsDecoderError;
use crate::metrics::MetricsChunk;
use crate::read::MetricsIterator;

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub path: &'a Path,
    entries: ReadDir,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;

        Ok(Self { path, entries })
    }
}

impl<'a> IntoIterator for DiagnosticData<'a> {
    type Item = Result<MetricsChunk, MetricsDecoderError>;

    type IntoIter = MetricsIterator;

    fn into_iter(self) -> Self::IntoIter {
        MetricsIterator::new(self.entries)
    }
}
