mod raw;

use std::fmt::Display;
use std::io::Cursor;
use std::io::Read;

use bson::Document;
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;

use crate::bytes;
use crate::compression;
use crate::error::MetricParseError;
use crate::metadata::Metadata;
use crate::metrics::raw::MetricParser;
use crate::metrics::raw::RawMetric;

#[derive(Debug, Clone)]
pub struct MetricsChunk {
    pub metadata: Metadata,
    pub metrics: Vec<Metric>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub groups: Vec<String>,
    pub measurements: Vec<Measurement>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Measurement {
    pub timestamp: DateTime<Utc>,
    pub value: MetricValue,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum MetricValue {
    UInt32(u32),
    Int32(i32),
    Int64(i64),
    Float64(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
}

impl From<MetricValue> for f64 {
    fn from(value: MetricValue) -> f64 {
        match value {
            MetricValue::UInt32(v) => v as f64,
            MetricValue::Int32(v) => v as f64,
            MetricValue::Int64(v) => v as f64,
            MetricValue::Float64(v) => v,
            MetricValue::Boolean(b) => b as u64 as f64,
            MetricValue::DateTime(dt) => dt.timestamp_millis() as f64,
        }
    }
}

impl Display for MetricValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetricValue::UInt32(n) => Display::fmt(n, f),
            MetricValue::Int32(n) => Display::fmt(n, f),
            MetricValue::Int64(n) => Display::fmt(n, f),
            MetricValue::Float64(n) => Display::fmt(n, f),
            MetricValue::Boolean(b) => Display::fmt(b, f),
            MetricValue::DateTime(dt) => Display::fmt(dt, f),
        }
    }
}

impl MetricsChunk {
    const METRIC_NAME_DELIMITER: &str = " ";
    const START_TIMESTAMP_METRIC_NAME: &str = "start";
    const END_TIMESTAMP_METRIC_NAME: &str = "end";

    pub(crate) fn from_reader<R: Read + ?Sized>(
        reader: &mut R,
    ) -> Result<MetricsChunk, MetricParseError> {
        let data = compression::decompress(reader)?;
        let mut cursor = Cursor::new(data.as_slice());

        let reference_doc = Document::from_reader(&mut cursor)?;
        let metrics_count: usize = bytes::read_le_u32(&mut cursor)?.try_into()?;
        let samples_count: usize = bytes::read_le_u32(&mut cursor)?.try_into()?;
        let metrics =
            MetricParser::parse(&reference_doc, &mut cursor, metrics_count, samples_count)?;

        MetricsChunk::from_raw(metrics, &reference_doc)
    }

    fn from_raw(
        metrics: Vec<RawMetric>,
        reference_doc: &Document,
    ) -> Result<MetricsChunk, MetricParseError> {
        let mut metrics_chunk: Vec<Metric> = Vec::with_capacity(metrics.len());
        let mut chunk_timestamps: Vec<DateTime<Utc>> = Vec::new();
        let mut timestamps: Vec<DateTime<Utc>> = Vec::new();

        for metric in metrics.into_iter() {
            if let Some(name) = metric.groups.last() {
                if name == Self::START_TIMESTAMP_METRIC_NAME {
                    let ts = to_timestamps(metric.values).collect();

                    if metric.groups.len() == 1 {
                        chunk_timestamps = ts;
                    } else {
                        timestamps = ts;
                    };

                    continue;
                }
            }

            if let Some(name) = metric.groups.last() {
                if name == Self::END_TIMESTAMP_METRIC_NAME {
                    continue;
                }
            }

            let name: String = metric.groups.join(Self::METRIC_NAME_DELIMITER);
            let measurements = timestamps
                .iter()
                .zip(metric.values)
                .map(|(start, value)| Measurement {
                    timestamp: start.to_owned(),
                    value: metric.vtype.convert(value),
                })
                .collect::<Vec<Measurement>>();

            let ts_err = || MetricParseError::MetricTimestampNotFound { name: name.clone() };
            let start_date = timestamps.first().ok_or_else(ts_err)?.to_owned();
            let end_date = timestamps.last().ok_or_else(ts_err)?.to_owned();

            metrics_chunk.push(Metric {
                name,
                groups: metric.groups,
                start_date,
                end_date,
                measurements,
            })
        }

        let ts_err = || MetricParseError::MetricTimestampNotFound {
            name: Self::START_TIMESTAMP_METRIC_NAME.to_owned(),
        };
        let start_chunk = chunk_timestamps.first().ok_or_else(ts_err)?.to_owned();
        let end_chunk = chunk_timestamps.last().ok_or_else(ts_err)?.to_owned();

        let metadata = Metadata::from_reference_document(reference_doc)?;

        Ok(MetricsChunk {
            start_date: start_chunk,
            end_date: end_chunk,
            metrics: metrics_chunk,
            metadata,
        })
    }
}

fn to_timestamps(values: Vec<u64>) -> impl Iterator<Item = DateTime<Utc>> {
    values.into_iter().map(|v| {
        Utc.timestamp_millis_opt(v as i64)
            .single()
            .expect("timestamp to be converted to UTC")
    })
}
