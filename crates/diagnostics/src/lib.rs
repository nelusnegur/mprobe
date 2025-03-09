mod bson;
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

use chrono::DateTime;
use chrono::Utc;

use crate::error::MetricParseError;
use crate::metrics::MetricsChunk;
use crate::read::MetricsIterator;

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub path: &'a Path,
    entries: ReadDir,
    filter: MetricsFilter,
}

impl<'a> DiagnosticData<'a> {
    pub fn new(path: &'a Path) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;
        let filter = MetricsFilter::default();

        Ok(Self {
            path,
            entries,
            filter,
        })
    }

    pub fn filter(path: &'a Path, filter: MetricsFilter) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;

        Ok(Self {
            path,
            entries,
            filter,
        })
    }
}

impl IntoIterator for DiagnosticData<'_> {
    type Item = Result<MetricsChunk, MetricParseError>;

    type IntoIter = MetricsIterator;

    fn into_iter(self) -> Self::IntoIter {
        MetricsIterator::new(self.entries, self.filter)
    }
}

pub struct DiagnosticDataBuilder<'a> {
    path: &'a Path,
    hostname: Option<String>,
    start_timestamp: Option<DateTime<Utc>>,
    end_timestamp: Option<DateTime<Utc>>,
}

impl<'a> DiagnosticDataBuilder<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self {
            path,
            hostname: None,
            start_timestamp: None,
            end_timestamp: None,
        }
    }

    pub fn host(&mut self, hostname: String) -> &mut Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn start_timestamp(&mut self, timestamp: DateTime<Utc>) -> &mut Self {
        self.start_timestamp = Some(timestamp);
        self
    }

    pub fn end_timestamp(&mut self, timestamp: DateTime<Utc>) -> &mut Self {
        self.end_timestamp = Some(timestamp);
        self
    }

    pub fn build(self) -> Result<DiagnosticData<'a>, io::Error> {
        let filter = MetricsFilter {
            hostname: self.hostname,
            start_timestamp: self.start_timestamp,
            end_timestamp: self.end_timestamp,
        };

        DiagnosticData::filter(self.path, filter)
    }
}

#[derive(Debug, Default)]
pub struct MetricsFilter {
    pub(crate) hostname: Option<String>,
    pub(crate) start_timestamp: Option<DateTime<Utc>>,
    pub(crate) end_timestamp: Option<DateTime<Utc>>,
}

impl MetricsFilter {
    pub fn new(
        hostname: Option<String>,
        start_timestamp: Option<DateTime<Utc>>,
        end_timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            hostname,
            start_timestamp,
            end_timestamp,
        }
    }
}
