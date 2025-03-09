use std::io;
use std::io::Cursor;
use std::io::Read;

use bson::spec::ElementType;
use bson::Document;
use chrono::TimeZone;
use chrono::Utc;

use crate::bytes;
use crate::error::MetricsDecoderError;
use crate::metrics::MetricValue;

pub(super) struct MetricParser;

impl MetricParser {
    pub fn parse(
        reference_doc: &Document,
        data: &mut Cursor<&[u8]>,
        metrics_count: usize,
        samples_count: usize,
    ) -> Result<Vec<RawMetric>, MetricsDecoderError> {
        let init_values = Self::read_initial_values(reference_doc, metrics_count)?;
        let metrics = Self::read_samples(data, init_values, samples_count)?;

        Ok(metrics)
    }

    fn read_initial_values(
        reference_doc: &Document,
        metrics_count: usize,
    ) -> Result<Vec<MetricInitVal>, MetricsDecoderError> {
        let mut metrics: Vec<MetricInitVal> = Vec::with_capacity(metrics_count);

        Self::select_metrics(reference_doc, Vec::new(), &mut metrics);

        if metrics.len() != metrics_count {
            return Err(MetricsDecoderError::MetricsCountMismatch);
        }

        Ok(metrics)
    }

    fn select_metrics(
        reference_doc: &Document,
        parent_key: Vec<String>,
        metrics: &mut Vec<MetricInitVal>,
    ) {
        for (key, value) in reference_doc {
            match value.element_type() {
                ElementType::Int32 => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::I32,
                        value.as_i32().unwrap() as u64,
                    ));
                }
                ElementType::Int64 => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::I64,
                        value.as_i64().unwrap() as u64,
                    ));
                }
                ElementType::Double => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::F64,
                        value.as_f64().unwrap() as u64,
                    ));
                }
                ElementType::Boolean => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::Bool,
                        value.as_bool().unwrap() as u64,
                    ));
                }
                ElementType::DateTime => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::DateTime,
                        value.as_datetime().unwrap().timestamp_millis() as u64,
                    ));
                }
                ElementType::Timestamp => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());
                    parts.push(String::from("time"));

                    metrics.push(MetricInitVal::new(
                        parts,
                        ValueType::U32,
                        value.as_timestamp().unwrap().time as u64,
                    ));

                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());
                    parts.push(String::from("increment"));

                    metrics.push(MetricInitVal::new(
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

                            Self::select_metrics(doc, parts, metrics);
                        }
                    }
                }
                ElementType::EmbeddedDocument => {
                    let mut parts = parent_key.clone();
                    parts.push(key.to_owned());

                    Self::select_metrics(value.as_document().unwrap(), parts, metrics);
                }
                _ => continue,
            }
        }
    }

    fn read_samples<R: Read + ?Sized>(
        reader: &mut R,
        metrics: Vec<MetricInitVal>,
        samples_count: usize,
    ) -> Result<Vec<RawMetric>, io::Error> {
        if samples_count == 0 {
            return Ok(metrics
                .into_iter()
                .map(|r| RawMetric::new(r.groups, r.vtype, vec![r.value]))
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

        let samples: Vec<RawMetric> = samples
            .into_iter()
            .zip(metrics)
            .map(|(s, m)| RawMetric::new(m.groups, m.vtype, s))
            .collect();

        Ok(samples)
    }
}

pub(super) struct RawMetric {
    pub(super) groups: Vec<String>,
    pub(super) vtype: ValueType,
    pub(super) values: Vec<u64>,
}

impl RawMetric {
    pub fn new(groups: Vec<String>, vtype: ValueType, values: Vec<u64>) -> Self {
        Self {
            groups,
            vtype,
            values,
        }
    }
}

struct MetricInitVal {
    groups: Vec<String>,
    value: u64,
    vtype: ValueType,
}

impl MetricInitVal {
    pub fn new(groups: Vec<String>, vtype: ValueType, value: u64) -> Self {
        Self {
            groups,
            value,
            vtype,
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum ValueType {
    U32,
    I32,
    I64,
    F64,
    Bool,
    DateTime,
}

impl ValueType {
    pub fn convert(&self, value: u64) -> MetricValue {
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
