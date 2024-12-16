use bson::Document;
use chrono::DateTime;
use chrono::Utc;

use crate::bson::DocumentKind;
use crate::bson::ReadDocument;
use crate::error::MetricsDecoderError;

#[derive(Debug, Default, Clone)]
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

    pub(crate) fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        match (self.start, self.end) {
            (None, None) => true,
            (None, Some(ref end)) => timestamp.le(end),
            (Some(ref start), None) => timestamp.ge(start),
            (Some(ref start), Some(ref end)) => timestamp.ge(start) && timestamp.le(end),
        }
    }
}

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
