use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::io::{Cursor, Read};

use bson::de;
use bson::spec::ElementType;
use bson::Document;
use chrono::{DateTime, TimeZone, Utc};

use crate::bytes;
use crate::compression;
use crate::error::MetricsDecoderError;
use crate::metadata::Metadata;

const METRIC_NAME_DELIMITER: char = ' ';

const START_TIMESTAMP_METRIC_NAME: &str = "start";
const END_TIMESTAMP_METRIC_NAME: &str = "end";

#[derive(Debug, Clone)]
pub struct MetricsChunk {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub metadata: Metadata,
    pub metrics: Vec<Metric>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub groups: Vec<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub measurements: Vec<Measurement>,
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

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ValueType {
    U32,
    I32,
    I64,
    F64,
    Bool,
    // TODO: Currently we use UTC time zone.
    // Check if it's possible to extract the exact time zone.
    DateTime,
}

impl ValueType {
    fn convert(&self, value: u64) -> MetricValue {
        match *self {
            ValueType::U32 => MetricValue::UInt32(value as u32),
            ValueType::I32 => MetricValue::Int32(value as i32),
            ValueType::I64 => MetricValue::Int64(value as i64),
            ValueType::F64 => MetricValue::Float64(value as f64),
            ValueType::Bool => MetricValue::Boolean(value != 0),
            ValueType::DateTime => MetricValue::DateTime(
                Utc.timestamp_millis_opt(value as i64)
                    .single()
                    .expect("timestamp to be converted to UTC"),
            ),
        }
    }
}

struct RawMetric {
    groups: Vec<String>,
    value: u64,
    vtype: ValueType,
}

impl RawMetric {
    pub fn new(groups: Vec<String>, vtype: ValueType, value: u64) -> Self {
        Self {
            groups,
            value,
            vtype,
        }
    }
}

// TODO: Fix me
// struct RawMetricChunk {
//     groups: Vec<String>,
//     vtype: ValueType,
//     value: Vec<u64>,
// }

impl MetricsChunk {
    pub(crate) fn from_reader<R: Read + ?Sized>(
        reader: &mut R,
    ) -> Result<MetricsChunk, MetricsDecoderError> {
        let data = compression::decompress(reader)?;
        let mut cursor = Cursor::new(data.as_slice());

        let reference_doc = MetricsChunk::read_reference_doc(&mut cursor)?;
        let metrics_count: usize = bytes::read_le_u32(&mut cursor)?.try_into()?;
        let samples_count: usize = bytes::read_le_u32(&mut cursor)?.try_into()?;

        let metrics = MetricsChunk::extract_metrics(&reference_doc, metrics_count)?;
        let metrics = MetricsChunk::read_samples(&mut cursor, metrics, samples_count)?;

        MetricsChunk::from(metrics, &reference_doc)
    }

    fn read_reference_doc<R: Read + ?Sized>(reader: &mut R) -> Result<Document, de::Error> {
        Document::from_reader(reader)
    }

    fn extract_metrics(
        reference_doc: &Document,
        metrics_count: usize,
    ) -> Result<Vec<RawMetric>, MetricsDecoderError> {
        let mut metrics: Vec<RawMetric> = Vec::with_capacity(metrics_count);

        MetricsChunk::select_metrics(reference_doc, Vec::new(), &mut metrics);

        if metrics.len() != metrics_count {
            return Err(MetricsDecoderError::MetricsCountMismatch);
        }

        Ok(metrics)
    }

    fn select_metrics(
        reference_doc: &Document,
        parent_key: Vec<String>,
        metrics: &mut Vec<RawMetric>,
    ) {
        for (key, value) in reference_doc {
            match value.element_type() {
                ElementType::Int32 => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::I32,
                        value.as_i32().unwrap() as u64,
                    ));
                }
                ElementType::Int64 => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::I64,
                        value.as_i64().unwrap() as u64,
                    ));
                }
                ElementType::Double => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::F64,
                        value.as_f64().unwrap() as u64,
                    ));
                }
                ElementType::Boolean => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::Bool,
                        value.as_bool().unwrap() as u64,
                    ));
                }
                ElementType::DateTime => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::DateTime,
                        value.as_datetime().unwrap().timestamp_millis() as u64,
                    ));
                }
                ElementType::Timestamp => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());
                    parts.push(String::from("time"));

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::U32,
                        value.as_timestamp().unwrap().time as u64,
                    ));

                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());
                    parts.push(String::from("increment"));

                    metrics.push(RawMetric::new(
                        parts,
                        ValueType::U32,
                        value.as_timestamp().unwrap().increment as u64,
                    ));
                }
                ElementType::Array => {
                    let array = value.as_array().unwrap();

                    for (idx, doc) in array.iter().enumerate() {
                        if let Some(doc) = doc.as_document() {
                            let mut parts = parent_key.clone();
                            parts.push(idx.to_string());

                            MetricsChunk::select_metrics(doc, parts, metrics);
                        }
                    }
                }
                ElementType::EmbeddedDocument => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    MetricsChunk::select_metrics(value.as_document().unwrap(), parts, metrics);
                }
                _ => continue,
            }
        }
    }

    fn read_samples<R: Read + ?Sized>(
        reader: &mut R,
        metrics: Vec<RawMetric>,
        samples_count: usize,
    ) -> Result<Vec<(Vec<String>, ValueType, Vec<u64>)>, io::Error> {
        if samples_count == 0 {
            return Ok(metrics
                .into_iter()
                .map(|r| (r.groups, r.vtype, vec![r.value]))
                .collect());
        }

        let metrics_count = metrics.len();
        let mut samples: Vec<Vec<u64>> = vec![vec![0; samples_count]; metrics_count];
        let mut zeroes_count: u64 = 0;

        for samples in samples.iter_mut() {
            for sample in samples.iter_mut() {
                if zeroes_count != 0 {
                    *sample = 0;
                    zeroes_count -= 1;
                    continue;
                }

                let delta = bytes::read_var_u64(reader)?;
                if delta == 0 {
                    let zero_count = bytes::read_var_u64(reader)?;
                    zeroes_count = zero_count;
                }

                *sample = delta;
            }
        }

        // Decode delta values
        for m in 0..metrics_count {
            samples[m][0] = samples[m][0].wrapping_add(metrics[m].value);

            for s in 1..samples_count {
                samples[m][s] = samples[m][s].wrapping_add(samples[m][s - 1]);
            }
        }

        let samples: Vec<(Vec<String>, ValueType, Vec<u64>)> = samples
            .into_iter()
            .zip(metrics)
            .map(|(s, m)| (m.groups, m.vtype, s))
            .collect();

        Ok(samples)
    }

    fn from(
        metrics: Vec<(Vec<String>, ValueType, Vec<u64>)>,
        reference_doc: &Document,
    ) -> Result<MetricsChunk, MetricsDecoderError> {
        let mut start_timestamp_metrics: HashMap<String, Vec<DateTime<Utc>>> = HashMap::new();
        let mut end_timestamp_metrics: HashMap<String, Vec<DateTime<Utc>>> = HashMap::new();

        for (name_parts, _, values) in metrics.iter() {
            if let Some(name) = name_parts.last() {
                if name == START_TIMESTAMP_METRIC_NAME {
                    let name = if name_parts.len() == 1 {
                        name.to_owned()
                    } else {
                        let first = name_parts.first().expect("name parts to be non empty");
                        metric_name(first, START_TIMESTAMP_METRIC_NAME)
                    };

                    start_timestamp_metrics.insert(
                        name,
                        values
                            .iter()
                            .map(|v| {
                                Utc.timestamp_millis_opt(*v as i64)
                                    .single()
                                    .expect("timestamp to be converted to UTC")
                            })
                            .collect(),
                    );
                } else if name == END_TIMESTAMP_METRIC_NAME {
                    let name = if name_parts.len() == 1 {
                        name.to_owned()
                    } else {
                        let first = name_parts.first().expect("name parts to be non empty");
                        metric_name(first, END_TIMESTAMP_METRIC_NAME)
                    };

                    end_timestamp_metrics.insert(
                        name,
                        values
                            .iter()
                            .map(|v| {
                                Utc.timestamp_millis_opt(*v as i64)
                                    .single()
                                    .expect("timestamp to be converted to UTC")
                            })
                            .collect(),
                    );
                }
            }
        }

        let mut metric_chunks: Vec<Metric> = Vec::with_capacity(
            metrics.len() - start_timestamp_metrics.len() - end_timestamp_metrics.len(),
        );

        let metrics_without_timestamps = metrics.iter().filter(|(m, _, _)| {
            m.last()
                .filter(|n| n.ends_with(START_TIMESTAMP_METRIC_NAME))
                .is_none()
                && m.last()
                    .filter(|n| n.ends_with(END_TIMESTAMP_METRIC_NAME))
                    .is_none()
        });

        for (metric, ty, values) in metrics_without_timestamps {
            // TODO: Use METRIC_NAME_DELIMITER constant
            let name: String = metric.join(" ");

            let collector = metric.first().expect("vector to not be empty");
            // .split(METRIC_NAME_DELIMITER)
            // .nth(1)
            // .ok_or(MetricsDecoderError::MetricCollectorNotFound)?;
            // let collector = metric_name("", collector);

            let start_metric_name = metric_name(collector, START_TIMESTAMP_METRIC_NAME);
            let end_metric_name = metric_name(collector, END_TIMESTAMP_METRIC_NAME);

            let start_timestamp_values = start_timestamp_metrics
                .get(start_metric_name.as_str())
                .ok_or_else(|| MetricsDecoderError::MetricNotFound {
                    name: start_metric_name.clone(),
                })?;
            let end_timestamp_values = end_timestamp_metrics
                .get(end_metric_name.as_str())
                .ok_or_else(|| MetricsDecoderError::MetricNotFound {
                    name: end_metric_name.clone(),
                })?;

            let measurements = start_timestamp_values
                .iter()
                .zip(values)
                .map(|(start, value)| Measurement {
                    timestamp: start.to_owned(),
                    value: ty.convert(*value),
                })
                .collect::<Vec<Measurement>>();

            let start_date =
                start_timestamp_values
                    .first()
                    .ok_or(MetricsDecoderError::MetricValueNotFound {
                        name: start_metric_name,
                    })?;

            let end_date =
                end_timestamp_values
                    .last()
                    .ok_or(MetricsDecoderError::MetricValueNotFound {
                        name: end_metric_name,
                    })?;

            metric_chunks.push(Metric {
                name,
                // TODO: See if we can get rid of cloning here
                groups: metric.clone(),
                start_date: start_date.to_owned(),
                end_date: end_date.to_owned(),
                measurements,
            })
        }

        let start_timestamp = start_timestamp_metrics
            .get(START_TIMESTAMP_METRIC_NAME)
            .and_then(|ts| ts.first())
            .ok_or_else(|| MetricsDecoderError::MetricNotFound {
                name: START_TIMESTAMP_METRIC_NAME.to_owned(),
            })?;
        let end_timestamp = end_timestamp_metrics
            .get(END_TIMESTAMP_METRIC_NAME)
            .and_then(|ts| ts.last())
            .ok_or_else(|| MetricsDecoderError::MetricNotFound {
                name: END_TIMESTAMP_METRIC_NAME.to_owned(),
            })?;

        let metadata = Metadata::from_reference_document(reference_doc)?;

        Ok(MetricsChunk {
            start_date: start_timestamp.to_owned(),
            end_date: end_timestamp.to_owned(),
            metrics: metric_chunks,
            metadata,
        })
    }
}

fn metric_name(parent_key: &str, key: &str) -> String {
    format!("{}{}{}", parent_key, METRIC_NAME_DELIMITER, key)
}
