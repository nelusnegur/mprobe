use mprobe_diagnostics::error::MetricParseError;
use mprobe_diagnostics::metrics::MetricsChunk;

pub(super) struct ErrorHandlingIter<I> {
    iter: I,
}

impl<I> ErrorHandlingIter<I>
where
    I: Iterator<Item = Result<MetricsChunk, MetricParseError>>,
{
    pub fn new(iter: I) -> ErrorHandlingIter<I> {
        Self { iter }
    }
}

impl<I> Iterator for ErrorHandlingIter<I>
where
    I: Iterator<Item = Result<MetricsChunk, MetricParseError>>,
{
    type Item = MetricsChunk;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Ok(item)) => return Some(item),
                Some(Err(err)) => {
                    // TODO: Log the error and what time range is missing
                    // We should also react accordingly to specific types of errors.
                    // For example, if the file from which we are reading was deleted,
                    // we should stop iterating.
                    //
                    // For now just print the error to the stderr
                    eprintln!("An error occurred while reading metrics: {:?}", err);
                    continue;
                }
                None => return None,
            }
        }
    }
}
