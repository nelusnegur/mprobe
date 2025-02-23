use std::fmt::Display;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::rc::Rc;

use chrono::DateTime;
use chrono::Utc;
use digest_auth::AuthContext;
use digest_auth::HttpMethod;
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use reqwest::blocking::Response;
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;

use crate::fetch::error::FetchError;
use crate::fetch::error::Result;

pub(crate) struct LogClient {
    client: Client,
    base_url: &'static str,
    credentials: Credentials,
    group_id: Rc<str>,
}

impl LogClient {
    pub fn new(group_id: Rc<str>, credentials: Credentials) -> Self {
        let base_url = "https://cloud.mongodb.com/api/atlas/v1.0";
        let client = Client::new();

        Self {
            client,
            base_url,
            credentials,
            group_id,
        }
    }

    pub fn create_job(&self, body: CreateJobBody) -> Result<CreatedJob> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs",
            base_url = self.base_url,
            group_id = self.group_id
        );
        let response = self
            .client
            .post(url)
            .json(&body)
            .digest_auth_send(&self.credentials)?;

        match response.status() {
            StatusCode::CREATED => {
                let job: CreatedJob = response.json()?;
                Ok(job)
            }
            status_code => {
                let message = response.text()?;
                let error = FetchError::Response {
                    status_code,
                    message,
                };
                Err(error)
            }
        }
    }

    pub fn get_job(&self, job_id: &str) -> Result<Job> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs/{job_id}",
            base_url = self.base_url,
            group_id = self.group_id,
            job_id = job_id
        );
        let response = self.client.get(url).digest_auth_send(&self.credentials)?;

        match response.status() {
            StatusCode::OK => {
                let job: Job = response.json()?;
                Ok(job)
            }
            status_code => {
                let message = response.text()?;
                let error = FetchError::Response {
                    status_code,
                    message,
                };
                Err(error)
            }
        }
    }

    pub fn download(&self, job_id: &str, path: &Path) -> Result<u64> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs/{job_id}/download",
            base_url = self.base_url,
            group_id = self.group_id,
            job_id = job_id
        );
        let mut response = self.client.get(url).digest_auth_send(&self.credentials)?;

        match response.status() {
            StatusCode::OK => {
                let mut writer = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)?;

                let bytes = io::copy(&mut response, &mut writer)?;
                Ok(bytes)
            }
            status_code => {
                let message = response.text()?;
                let error = FetchError::Response {
                    status_code,
                    message,
                };
                Err(error)
            }
        }
    }
}

pub(crate) struct Credentials {
    pub api_key: String,
    pub api_secret: String,
}

impl Credentials {
    pub(crate) fn new(api_key: String, api_secret: String) -> Self {
        Self {
            api_key,
            api_secret,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum Resource {
    Cluster,
    ReplicaSet,
    Process,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Resource::Cluster => write!(f, "cluster"),
            Resource::ReplicaSet => write!(f, "replica set"),
            Resource::Process => write!(f, "process"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum LogType {
    Ftdc,
    Mongodb,
    MonitoringAgent,
    AutomationAgent,
    BackupAgent,
}

impl Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LogType::Ftdc => write!(f, "FTDC"),
            LogType::Mongodb => write!(f, "MongoDB"),
            LogType::MonitoringAgent => write!(f, "Monitoring agent"),
            LogType::AutomationAgent => write!(f, "Automation agent"),
            LogType::BackupAgent => write!(f, "Backup agent"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateJobBody<'n> {
    resource_type: Resource,
    resource_name: &'n str,
    size_requested_per_file_bytes: u64,
    log_types: Vec<LogType>,
    redacted: bool,
    log_collection_from_date: Option<DateTime<Utc>>,
    log_collection_to_date: Option<DateTime<Utc>>,
}
impl<'n> CreateJobBody<'n> {
    pub(crate) fn new(
        resource_type: Resource,
        resource_name: &'n str,
        size_requested_per_file_bytes: u64,
        log_types: Vec<LogType>,
        redacted: bool,
        log_collection_from_date: Option<DateTime<Utc>>,
        log_collection_to_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            resource_type,
            resource_name,
            size_requested_per_file_bytes,
            log_types,
            redacted,
            log_collection_from_date,
            log_collection_to_date,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreatedJob {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Job {
    pub id: String,
    pub status: JobStatus,
    pub root_resource_type: Resource,
    pub root_resource_name: String,
    pub resource_type: Resource,
    pub resource_name: String,
    pub creation_date: DateTime<Utc>,
    pub expiration_date: DateTime<Utc>,
    pub log_types: Vec<LogType>,
    pub redacted: bool,
    pub size_requested_per_file_bytes: u64,
    pub uncompressed_size_total_bytes: u64,
    pub log_collection_from_date: Option<DateTime<Utc>>,
    pub log_collection_to_date: Option<DateTime<Utc>>,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Server job")?;
        writeln!(f, "id: {id}", id = self.id)?;
        writeln!(f, "status: {status}", status = self.status)?;

        writeln!(
            f,
            "resource: {name} {rtype}",
            name = self.resource_name,
            rtype = self.resource_type
        )?;

        writeln!(
            f,
            "root resource: {name} {rtype}",
            name = self.root_resource_name,
            rtype = self.root_resource_type
        )?;

        writeln!(f, "created at {date}", date = self.creation_date)?;
        writeln!(f, "expires at {date}", date = self.expiration_date)?;

        let logs = self
            .log_types
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(",");

        writeln!(f, "requested logs: {logs}")?;
        writeln!(f, "redacted: {redacted}", redacted = self.redacted)?;

        if let Some(from_date) = self.log_collection_from_date {
            writeln!(f, "created at {from_date}")?;
        }

        if let Some(to_date) = self.log_collection_to_date {
            writeln!(f, "created at {to_date}")?;
        }

        writeln!(
            f,
            "requested {bytes} bytes per file",
            bytes = self.size_requested_per_file_bytes
        )?;

        if self.uncompressed_size_total_bytes > 0 {
            writeln!(
                f,
                "total uncompressed {bytes} bytes",
                bytes = self.uncompressed_size_total_bytes
            )?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum JobStatus {
    Success,
    Failure,
    InProgress,
    MarkedForExpiry,
    Expired,
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            JobStatus::Success => write!(f, "success"),
            JobStatus::Failure => write!(f, "failure"),
            JobStatus::InProgress => write!(f, "in progress"),
            JobStatus::MarkedForExpiry => write!(f, "marked for expiry"),
            JobStatus::Expired => write!(f, "expired"),
        }
    }
}

trait DigestAuth {
    const WWW_AUTHENTICATE_HEADER: &str = "www-authenticate";

    fn digest_auth_send(self, credentials: &Credentials) -> Result<Response>;
}

impl DigestAuth for RequestBuilder {
    fn digest_auth_send(self, credentials: &Credentials) -> Result<Response> {
        let request_builder = self
            .try_clone()
            .expect("request builder to be clonable for digest auth");

        let response = self.send()?;

        match response.status() {
            StatusCode::UNAUTHORIZED => {
                let headers = response.headers();
                let Some(www_auth) = headers.get(Self::WWW_AUTHENTICATE_HEADER) else {
                    let message = format!("Digest authentication failed: the server did not include the {header} header in the response.", header = Self::WWW_AUTHENTICATE_HEADER);
                    return Err(FetchError::DigestAuth(message));
                };

                // Create the Request object to read the necessary information
                // required for the digest authentication, without cloning it.
                let (client, request) = request_builder.build_split();
                let request = request?;

                let auth_context = AuthContext::new_with_method(
                    &credentials.api_key,
                    &credentials.api_secret,
                    request.url().path(),
                    request.body().and_then(|b| b.as_bytes()),
                    HttpMethod::from(request.method().as_str()),
                );
                let mut prompt = digest_auth::parse(www_auth.to_str()?)?;
                let header = prompt.respond(&auth_context)?.to_header_string();

                let request_builder = RequestBuilder::from_parts(client, request);
                let response = request_builder.header(AUTHORIZATION, header).send()?;

                Ok(response)
            }
            status if status.is_success() => Ok(response),
            status => {
                let message = response.text()?;
                let error = FetchError::DigestAuth(format!(
                    "Digest authentication failed with status code {status}. {message}",
                ));
                Err(error)
            }
        }
    }
}
