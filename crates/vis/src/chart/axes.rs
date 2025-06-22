use mprobe_diagnostics::metrics::Measurement;
use mprobe_diagnostics::metrics::MetricValue;

use serde::Serialize;

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum AxisType {
    #[default]
    Linear,
    Date,
    Log,
    Category,
    Multicategory,
}

impl AxisType {
    pub fn yaxis(measurements: &[Measurement]) -> AxisType {
        if let Some(first_measurement) = measurements.first() {
            match first_measurement.value {
                MetricValue::DateTime(_) => AxisType::Date,
                MetricValue::UInt32(_)
                | MetricValue::Int32(_)
                | MetricValue::Int64(_)
                | MetricValue::Float64(_)
                | MetricValue::Boolean(_) => AxisType::Linear,
            }
        } else {
            AxisType::Linear
        }
    }
}
