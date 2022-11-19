use bson::Document;

use crate::diagnostics::error::KeyValueAccessError;
use crate::diagnostics::error::ValueAccessResultExt;

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
    pub(crate) fn from_reference_document(doc: &Document) -> Result<Metadata, KeyValueAccessError> {
        let server_status = doc
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
