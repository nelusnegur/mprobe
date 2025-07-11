//! A library for parsing and reading MongoDB diagnostic data,
//! generated by the Full Time Diagnostic Data Capture (FTDC) mechanism.
//!
//! # Read the diagnostic data
//!
//! One can read the diagnostic data by constructing an instance of [DiagnosticData],
//! and iterate over it to get the diagnostic metrics. The iterator yields one
//! [chunk] of the diagnostic data at a time. Each chunk contains a list of [metrics]
//! and [metadata] associated with it. The metrics are always returned sorted in
//! ascending order.
//!
//! [chunk]: crate::metrics::MetricsChunk
//! [metadata]: crate::metadata::Metadata
//! [metrics]: crate::metrics::Metric
//!
//! The following example shows how to read through the diagnostic data and print
//! the metric names on the standard output.
//!
//! ```no_run
//! use std::path::Path;
//! use std::result::Result;
//!
//! use mprobe_diagnostics::DiagnosticData;
//! use mprobe_diagnostics::error::MetricParseError;
//!
//! fn main() -> Result<(), MetricParseError> {
//!     // Note: this example needs a valid path
//!     // that contains diagnostic data for it to run
//!     let path = Path::new("/path/to/diagnostic/data");
//!     let diagnostic_data = DiagnosticData::new(&path)?;
//!
//!     for chunk in diagnostic_data {
//!         for metric in chunk?.metrics {
//!             println!("{}", metric.name);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Filter the diagnostic data
//!
//! Since the [DiagnosticData] implements [IntoIterator], one could use
//! the [Iterator::filter] combinator to filter the diagnostic data. However that will
//! be applied to all the diagnostic data contained in the provided path, which
//! can contain a lot of data. When one needs only the diagnostic data of a node,
//! process, or the data in a specific time window, one can use
//! the [DiagnosticData::filter] function and provide an instance of [MetricsFilter].
//!
//! In the example below it is show how one could use it.
//!
//! ```no_run
//! use std::path::Path;
//! use chrono::{DateTime, Duration, TimeDelta, Utc};
//! use mprobe_diagnostics::{DiagnosticData, MetricsFilter};
//!
//! let path = Path::new("/path/to/diagnostic/data");
//!
//! let node = String::from("node-1");
//! let start = Utc::now() - Duration::hours(1);
//! let end = Utc::now();
//!
//! let filter = MetricsFilter::new(Some(node), Some(start), Some(end));
//! let diagnostic_data = DiagnosticData::filter(&path, filter).expect("valid path");
//! ```
//!

#![warn(missing_docs)]

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

/// `DiagnosticData` defines an API for parsing and reading MongoDB diagnostic data.
#[derive(Debug)]
pub struct DiagnosticData {
    entries: ReadDir,
    filter: MetricsFilter,
}

impl DiagnosticData {
    /// Creates a new `DiagnosticData` that will parse and read
    /// the diagnostic data at the specified `path`.
    ///
    /// The `path` must be valid and point to a directory containing
    /// the diagnostic data unarchived.
    pub fn new(path: &Path) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;
        let filter = MetricsFilter::default();

        Ok(Self { entries, filter })
    }

    /// Creates a new `DiagnosticData` that will parse and read
    /// the diagnostic data at the specified `path` and filter it
    /// according to the `filter` specification.
    ///
    /// The `path` must be valid and point to a directory containing
    /// the diagnostic data unarchived.
    pub fn filter(path: &Path, filter: MetricsFilter) -> Result<Self, io::Error> {
        let entries = fs::read_dir(path)?;

        Ok(Self { entries, filter })
    }
}

impl IntoIterator for DiagnosticData {
    type Item = Result<MetricsChunk, MetricParseError>;

    type IntoIter = MetricsIterator;

    fn into_iter(self) -> Self::IntoIter {
        MetricsIterator::new(self.entries, self.filter)
    }
}

/// `MetricsFilter` specifies a filter for the diagnostic data.
#[derive(Debug, Default)]
pub struct MetricsFilter {
    pub(crate) hostname: Option<String>,
    pub(crate) start: Option<DateTime<Utc>>,
    pub(crate) end: Option<DateTime<Utc>>,
}

impl MetricsFilter {
    /// Creates a new `MetricsFilter` used for filtering diagnostic data.
    ///
    /// The `MetricsFilter` enables filtering the data based on the following
    /// parameters:
    ///
    /// * `hostname` - if set, selects the data belonging for
    ///   the specified hostname;
    /// * `start` - if set, selects the data starting at the specified timestamp;
    /// * `end` - if set, selects the data up until the specified timestamp.
    pub fn new(
        hostname: Option<String>,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            hostname,
            start,
            end,
        }
    }
}
