use std::rc::Rc;

use bson::Document;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use crate::bson::DocumentKind;
use crate::bson::ReadDocument;
use crate::error::MetricParseError;

#[derive(Debug, Default)]
pub(crate) struct TimeWindow {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
}

impl TimeWindow {
    pub fn new(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        Self { start, end }
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

    pub(crate) fn overlaps(&self, start: &DateTime<Utc>, end: &DateTime<Utc>) -> bool {
        match (self.start, self.end) {
            (None, None) => true,
            (Some(ref s), None) => end.ge(s),
            (None, Some(ref e)) => start.le(e),
            (Some(ref s), Some(ref e)) => start.le(e) && end.ge(s),
        }
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
pub(crate) struct TimeWindowFilter<I> {
    iter: I,
    time_window: Rc<TimeWindow>,
    time_margin: Duration,
}

impl<I> TimeWindowFilter<I>
where
    I: Iterator<Item = Result<Document, MetricParseError>>,
{
    pub fn new(iter: I, time_window: Rc<TimeWindow>) -> Self {
        Self {
            iter,
            time_window,
            time_margin: Duration::hours(2),
        }
    }
}

impl<I> Iterator for TimeWindowFilter<I>
where
    I: Iterator<Item = Result<Document, MetricParseError>>,
{
    type Item = Result<Document, MetricParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Ok(doc) => match doc.timestamp() {
                    Ok(ts) => {
                        if self.time_window.includes_with_margin(&ts, self.time_margin) {
                            return Some(Ok(doc));
                        }
                    }
                    Err(err) => return Some(Err(err)),
                },
                item => return Some(item),
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
    I: Iterator<Item = Result<Document, MetricParseError>>,
{
    type Item = Result<Document, MetricParseError>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next()? {
                Ok(doc) => match doc.kind() {
                    Ok(DocumentKind::Metadata) => match doc.hostname() {
                        Ok(hostname) => {
                            self.are_host_metrics =
                                self.hostname.as_ref().is_none_or(|hn| hn == hostname);

                            if self.are_host_metrics {
                                return Some(Ok(doc));
                            }
                        }
                        Err(err) => return Some(Err(err)),
                    },
                    Ok(_) => {
                        if self.are_host_metrics {
                            return Some(Ok(doc));
                        }
                    }
                    Err(err) => return Some(Err(err)),
                },
                doc => return Some(doc),
            }
        }
    }
}
