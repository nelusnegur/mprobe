use bson::Document;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::bson::DocumentKind;
use crate::bson::ReadDocument;
use crate::error::MetricsDecoderError;

#[derive(Debug, Default)]
pub(crate) struct TimeWindow {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

impl TimeWindow {
    pub fn new(
        start_timestamp: Option<DateTime<Utc>>,
        end_timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            start: start_timestamp,
            end: end_timestamp,
        }
    }

    pub(crate) fn includes(&self, timestamp: &DateTime<Utc>) -> bool {
        self.includes_with_margin(timestamp, Duration::zero())
    }

    pub(crate) fn includes_with_margin(&self, timestamp: &DateTime<Utc>, margin: Duration) -> bool {
        match (self.start, self.end) {
            (None, None) => true,
            (Some(start), None) => timestamp.ge(&(start - margin)),
            (None, Some(end)) => timestamp.le(&(end + margin)),
            (Some(start), Some(end)) => {
                timestamp.ge(&(start - margin)) && timestamp.le(&(end + margin))
            }
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct HostnameFilter<I> {
    iter: I,
    hostname: Option<String>,
    are_host_metrics: bool,
}

impl<I> HostnameFilter<I> {
    pub(crate) fn new(iter: I, hostname: Option<String>) -> Self {
        Self {
            iter,
            hostname,
            are_host_metrics: false,
        }
    }
}

impl<I> Iterator for HostnameFilter<I>
where
    I: Iterator<Item = Result<Document, MetricsDecoderError>>,
{
    type Item = Result<Document, MetricsDecoderError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(item) => match item {
                    Ok(ref doc) => match doc.kind() {
                        Ok(DocumentKind::Metadata) => match doc.hostname() {
                            Ok(hostname) => {
                                self.are_host_metrics =
                                    self.hostname.as_ref().is_none_or(|hn| hn == hostname);

                                if self.are_host_metrics {
                                    return Some(item);
                                }
                            }
                            Err(err) => return Some(Err(err)),
                        },
                        Ok(_) => {
                            if self.are_host_metrics {
                                return Some(item);
                            }
                        }
                        Err(err) => return Some(Err(err)),
                    },
                    doc => return Some(doc),
                },
                None => return None,
            }
        }
    }
}
