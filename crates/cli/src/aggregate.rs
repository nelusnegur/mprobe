use mprobe::diagnostics::error::MetricsDecoderError;
use mprobe::diagnostics::metrics::MetricValue;
use mprobe::diagnostics::metrics::MetricsChunk;

pub(crate) struct AggregateMetricsIter<I> {
    metric_chunks: I,
    metrics: Option<std::vec::IntoIter<(f64, f64)>>,
}

impl<I> Iterator for AggregateMetricsIter<I>
where
    I: Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>,
{
    type Item = (f64, f64);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.metrics {
                Some(ref mut metrics) => match metrics.next() {
                    None => self.metrics = None,
                    item => return item,
                },
                None => match self.metric_chunks.next()? {
                    Ok(chunk) if chunk.metadata.host.contains("4014e34491b5") => {
                        let inner_iter = chunk
                            .metrics
                            .into_iter()
                            .filter(|m| {
                                m.name.contains(
                                    "/serverStatus/wiredTiger/cache/bytes read into cache",
                                )
                            })
                            .flat_map(|m| m.measurements)
                            .map(|m| (convert(m.value), m.timestamp.timestamp() as f64))
                            .collect::<Vec<(f64, f64)>>();

                        self.metrics = Some(inner_iter.into_iter());
                    }
                    Ok(_) => continue,
                    Err(_) => todo!(),
                },
            }
        }
    }
}

impl<I> AggregateMetricsIter<I>
where
    I: Iterator<Item = Result<MetricsChunk, MetricsDecoderError>>,
{
    pub fn new(metric_chunks: I) -> AggregateMetricsIter<I> {
        Self {
            metric_chunks,
            metrics: None,
        }
    }
}

// TODO: Remove this temporary function
fn convert(value: MetricValue) -> f64 {
    match value {
        MetricValue::UInt32(v) => v as f64,
        MetricValue::Int32(v) => v as f64,
        MetricValue::Int64(v) => v as f64,
        MetricValue::Float64(v) => v,
        MetricValue::Boolean(b) => b as u64 as f64,
        MetricValue::DateTime(dt) => dt.timestamp_millis() as f64,
    }
}
