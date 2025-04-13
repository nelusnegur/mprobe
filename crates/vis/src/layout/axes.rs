use mprobe_diagnostics::metrics::Measurement;
use mprobe_diagnostics::metrics::MetricValue;

use crate::chart::AxisType;

pub(crate) struct ChartLayout;

impl ChartLayout {
    pub fn yaxis(_metric_name: &str, measurements: &[Measurement]) -> AxisType {
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
