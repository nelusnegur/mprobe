mod compression;
mod filter;
mod read;

pub mod error;
pub mod metadata;
pub mod metrics;

use std::fs;
use std::fs::ReadDir;
use std::io;
use std::path::Path;

use crate::diagnostics::error::MetricParserError;
use crate::diagnostics::metrics::MetricsChunk;
use crate::diagnostics::read::MetricsIterator;

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
    type Item = Result<MetricsChunk, MetricParserError>;

    type IntoIter = MetricsIterator;

    fn into_iter(self) -> Self::IntoIter {
        MetricsIterator::new(self.entries)
    }
}
