use bson::Document;
use chrono::DateTime;
use chrono::Utc;

use crate::error::MetricsDecoderError;
use crate::error::ValueAccessResultExt;

const DATA_TYPE_KEY: &str = "type";
const METRICS_CHUNK_DATA_TYPE: i32 = 1;

const ID_KEY: &str = "_id";
const METRICS_DOC_KEY: &str = "doc";

const HOST_INFO_KEY: &str = "hostInfo";
const SYSTEM_KEY: &str = "system";
const HOSTNAME_KEY: &str = "hostname";

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

pub(crate) fn metrics_chunk(document: &Document) -> Result<bool, MetricsDecoderError> {
    document
        .get_i32(DATA_TYPE_KEY)
        .map(|dt| dt == METRICS_CHUNK_DATA_TYPE)
        .map_value_access_err(DATA_TYPE_KEY)
        .map_err(MetricsDecoderError::from)
}

pub(crate) fn timestamp(
    document: &Document,
    filter: &TimeWindow,
) -> Result<bool, MetricsDecoderError> {
    document
        .get_datetime(ID_KEY)
        .map_value_access_err(ID_KEY)
        .map(|ts| filter.contains(ts.to_chrono()))
        .map_err(MetricsDecoderError::from)
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

    fn hostname(document: &Document) -> Result<&str, MetricsDecoderError> {
        let host_info = document
            .get_document(HOST_INFO_KEY)
            .map_value_access_err(HOST_INFO_KEY)?;

        let system = host_info
            .get_document(SYSTEM_KEY)
            .map_value_access_err(SYSTEM_KEY)?;

        let hostname = system
            .get_str(HOSTNAME_KEY)
            .map_value_access_err(HOSTNAME_KEY)?;

        Ok(hostname)
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
                    // TODO: Filter first on document type given that in each document
                    // first item is the metadata and it's a rare occurrence
                    Ok(ref doc) => match doc.get_document(METRICS_DOC_KEY) {
                        Ok(doc) => match Self::hostname(doc) {
                            Ok(hostname) => {
                                self.are_host_metrics =
                                    self.hostname.as_ref().is_none_or(|hn| hn == hostname);
                                continue;
                            }
                            Err(err) => return Some(Err(err)),
                        },
                        _ if self.are_host_metrics => return Some(item),
                        _ => continue,
                    },
                    err @ Err(_) => return Some(err),
                },
                None => return None,
            }
        }
    }
}
