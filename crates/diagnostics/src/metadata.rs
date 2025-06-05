use bson::Document;

use crate::error::KeyAccessError;
use crate::error::ValueAccessResultExt;

const COMMON_KEY: &str = "common";
const SERVER_STATUS_KEY: &str = "serverStatus";
const HOST_KEY: &str = "host";
const PROCESS_KEY: &str = "process";
const VERSION_KEY: &str = "version";

#[derive(Debug, Clone)]
pub struct Metadata {
    pub host: String,
    pub process: String,
    pub version: String,
}

impl Metadata {
    pub(crate) fn from_reference_document(doc: &Document) -> Result<Metadata, KeyAccessError> {
        // In MongoDB 8.0 a new nested field, common, was introduced,
        // and we have to account for it as well until all the previous
        // versions are no longer supported.
        let common = match doc.get_document(COMMON_KEY) {
            Ok(common) => common,
            Err(_) => doc,
        };

        let server_status = common
            .get_document(SERVER_STATUS_KEY)
            .map_value_access_err(SERVER_STATUS_KEY)?;

        let metadata = Self {
            host: server_status
                .get_str(HOST_KEY)
                .map_value_access_err(HOST_KEY)?
                .to_owned(),
            process: server_status
                .get_str(PROCESS_KEY)
                .map_value_access_err(PROCESS_KEY)?
                .to_owned(),
            version: server_status
                .get_str(VERSION_KEY)
                .map_value_access_err(VERSION_KEY)?
                .to_owned(),
        };

        Ok(metadata)
    }
}
