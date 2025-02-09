use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::rc::Rc;

use chrono::DateTime;
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;

pub(crate) struct LogClient {
    client: Client,
    base_url: &'static str,
    credentials: Credentials,
    group_id: Rc<str>,
}

impl LogClient {
    pub fn new(group_id: Rc<str>, credentials: Credentials) -> Self {
        let base_url = "https://cloud.mongodb.com/api/public/v1.0";
        let client = Client::new();

        Self {
            client,
            base_url,
            credentials,
            group_id,
        }
    }

    pub fn create_job(&self, body: CreateJobBody) -> Result<JobId> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs",
            base_url = self.base_url,
            group_id = self.group_id
        );
        let body = serde_json::to_string(&body)?;
        let response = self.client.post(url).body(body).send()?;

        match response.status() {
            StatusCode::CREATED => {
                let job_id: JobId = response.json()?;
                Ok(job_id)
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

    pub fn get_job(&self, job_id: JobId) -> Result<Job> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs/{job_id}",
            base_url = self.base_url,
            group_id = self.group_id,
            job_id = job_id.0
        );
        let response = self.client.get(url).send()?;

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

    pub fn download(&self, job_id: JobId, path: &Path) -> Result<u64> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs/{job_id}/download",
            base_url = self.base_url,
            group_id = self.group_id,
            job_id = job_id.0
        );
        let mut response = self.client.get(url).send()?;

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
    api_key: String,
    api_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub(crate) enum Resource {
    Cluster,
    ReplicaSet,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateJobBody {
    resource_type: Resource,
    resource_name: String,
    size_requested_per_file_bytes: u64,
    log_types: Vec<LogType>,
    redacted: bool,
    log_collection_from_date: Option<DateTime<Utc>>,
    log_collection_to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub(crate) struct JobId(String);

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct Job {
    id: String,
    status: JobStatus,
    resource_type: Resource,
    resource_name: String,
    creation_date: DateTime<Utc>,
    expiration_date: DateTime<Utc>,
    log_types: Vec<LogType>,
    redacted: bool,
    size_requested_per_file_bytes: u64,
    uncompressed_size_total_bytes: u64,
    log_collection_from_date: Option<DateTime<Utc>>,
    log_collection_to_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum JobStatus {
    Success,
    Failure,
    InProgress,
    MarkedForExpiry,
    Expired,
}

pub type Result<T> = std::result::Result<T, FetchError>;

#[derive(Debug)]
pub(crate) enum FetchError {
    Http(reqwest::Error),
    Response {
        status_code: StatusCode,
        message: String,
    },
    Json(serde_json::Error),
    Io(io::Error),
}

impl From<reqwest::Error> for FetchError {
    fn from(error: reqwest::Error) -> Self {
        FetchError::Http(error)
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(error: serde_json::Error) -> Self {
        FetchError::Json(error)
    }
}

impl From<io::Error> for FetchError {
    fn from(error: io::Error) -> Self {
        FetchError::Io(error)
    }
}
