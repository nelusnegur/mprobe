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

use crate::fetch::error::Result;
use crate::fetch::error::FetchError;

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
        let response = self
            .client
            .post(url)
            .json(&body)
            .digest_auth_send(&self.credentials)?;

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

    pub fn download(&self, job_id: JobId, path: &Path) -> Result<u64> {
        let url = format!(
            "{base_url}/groups/{group_id}/logCollectionJobs/{job_id}/download",
            base_url = self.base_url,
            group_id = self.group_id,
            job_id = job_id.0
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
