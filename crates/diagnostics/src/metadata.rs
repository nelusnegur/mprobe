//! Defines an API for reading the metadata associated with the diagnostic metrics.

use bson::Document;

use crate::error::KeyAccessError;
use crate::error::ValueAccessResultExt;

/// `Metadata` defines the metadata associated with the diagnostic metrics.
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Specifies the host name of the node that generated the diagnostic metrics.
    pub host: String,

    /// Specifies the process, e.g. mongod or mongos, that generated
    /// the diagnostic metrics.
    pub process: String,

    /// Specifies the database version on the node.
    pub version: String,
}

impl Metadata {
    const COMMON_KEY: &str = "common";
    const SERVER_STATUS_KEY: &str = "serverStatus";
    const HOST_KEY: &str = "host";
    const PROCESS_KEY: &str = "process";
    const VERSION_KEY: &str = "version";

    pub(crate) fn from_reference_document(doc: &Document) -> Result<Metadata, KeyAccessError> {
        // In MongoDB 8.0 a new nested field, common, was introduced,
        // and we have to account for it as well until all the previous
        // versions are no longer supported.
        let common = match doc.get_document(Self::COMMON_KEY) {
            Ok(common) => common,
            Err(_) => doc,
        };

        let server_status = common
            .get_document(Self::SERVER_STATUS_KEY)
            .map_value_access_err(Self::SERVER_STATUS_KEY)?;

        let metadata = Self {
            host: server_status
                .get_str(Self::HOST_KEY)
                .map_value_access_err(Self::HOST_KEY)?
                .to_owned(),
            process: server_status
                .get_str(Self::PROCESS_KEY)
                .map_value_access_err(Self::PROCESS_KEY)?
                .to_owned(),
            version: server_status
                .get_str(Self::VERSION_KEY)
                .map_value_access_err(Self::VERSION_KEY)?
                .to_owned(),
        };

        Ok(metadata)
    }
}
