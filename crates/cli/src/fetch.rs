mod client;
pub(crate) mod error;

use std::path::PathBuf;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

use crate::cli;
use crate::cli::FetchArgs;
use crate::cli::PathExt;
use crate::error::CliError;
use crate::fetch::client::CreateJobBody;
use crate::fetch::client::Credentials;
use crate::fetch::client::JobStatus;
use crate::fetch::client::LogClient;
use crate::fetch::client::LogType;
use crate::fetch::client::Resource;
use crate::fetch::error::FetchError;
use crate::fetch::error::JobErrorStatus;

pub(crate) fn fetch(args: FetchArgs) -> Result<(), CliError> {
    let credentials = Credentials::new(args.api_key, args.api_secret);
    let client = LogClient::new(Rc::from(args.project), credentials);
    let path = args.path.or_current_dir()?;

    let fetcher = MetricsFetch::new(
        client,
        path,
        args.resource_type.into(),
        args.resource_name,
        args.from,
        args.to,
        args.redacted,
    );

    Ok(fetcher.fetch()?)
}

pub(crate) struct MetricsFetch {
    client: LogClient,
    path: PathBuf,
    resource_type: Resource,
    resource_name: String,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    redacted: bool,
    size_requested_per_file_bytes: u64,
}

impl MetricsFetch {
    const ARCHIVE_NAME: &'static str = "diagnostic_data.tar.gz";

    pub fn new(
        client: LogClient,
        path: PathBuf,
        resource_type: Resource,
        resource_name: String,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        redacted: bool,
    ) -> Self {
        let path = path.join(Self::ARCHIVE_NAME);

        Self {
            client,
            path,
            resource_type,
            resource_name,
            from,
            to,
            redacted,
            size_requested_per_file_bytes: 10000000,
        }
    }

    pub fn fetch(&self) -> Result<(), FetchError> {
        let body = CreateJobBody::new(
            self.resource_type,
            &self.resource_name,
            self.size_requested_per_file_bytes,
            vec![LogType::Ftdc],
            self.redacted,
            self.from,
            self.to,
        );

        let job = self.client.create_job(body)?;
        let job = self.client.get_job(&job.id)?;

        println!("The background job is created. {job}");

        loop {
            let job = self.client.get_job(&job.id)?;

            match job.status {
                JobStatus::Success => {
                    self.client.download(&job.id, &self.path)?;
                    return Ok(());
                }
                JobStatus::InProgress => {
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
                JobStatus::Failure => {
                    return Err(FetchError::Job(JobErrorStatus::Failure));
                }
                JobStatus::MarkedForExpiry => {
                    return Err(FetchError::Job(JobErrorStatus::MarkedForExpiry));
                }
                JobStatus::Expired => {
                    return Err(FetchError::Job(JobErrorStatus::Expired));
                }
            }
        }
    }
}

impl From<cli::Resource> for Resource {
    fn from(resource: cli::Resource) -> Self {
        match resource {
            cli::Resource::Cluster => Resource::Cluster,
            cli::Resource::ReplicaSet => Resource::ReplicaSet,
            cli::Resource::Process => Resource::Process,
        }
    }
}
