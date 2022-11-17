use bson::{document::ValueAccessResult, Document};

#[derive(Debug, Clone)]
pub struct Metadata {
    pub host: String,
    pub process: String,
    pub version: String,
}

impl Metadata {
    pub(crate) fn from_reference_document(doc: &Document) -> ValueAccessResult<Metadata> {
        let server_status = doc.get_document("serverStatus")?;
        let metadata = Self {
            host: server_status.get_str("host")?.to_owned(),
            process: server_status.get_str("process")?.to_owned(),
            version: server_status.get_str("version")?.to_owned(),
        };

        Ok(metadata)
    }
}
