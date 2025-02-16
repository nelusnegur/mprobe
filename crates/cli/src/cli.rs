use std::env;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;

use crate::error::CliError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Visualize the diagnostic data and generate a visual representation of it.
    View {
        /// Specify the path from where to read the diagnostic data.
        /// The path must exist and it must point to a directory.
        #[arg(short, long, value_parser(parse_path))]
        path: PathBuf,

        /// Specify the path where the generated output will be created.
        /// If the output path is not specified then the current working
        /// directory is used.
        #[arg(short, long, value_parser(parse_path))]
        output_path: Option<PathBuf>,

        /// Filter metrics by the host name.
        #[arg(short, long)]
        node: Option<String>,

        /// Specify the start timestamp of the metrics.
        #[arg(short, long)]
        start_timestamp: Option<DateTime<Utc>>,

        /// Specify the end timestamp of the metrics.
        #[arg(short, long)]
        end_timestamp: Option<DateTime<Utc>>,
    },
    /// Fetch the diagnostic data from the Cloud Manager.
    Fetch(FetchArgs),
}

#[derive(Args)]
pub(crate) struct FetchArgs {
    /// The project id of the Cloud Manager.
    #[arg(short, long)]
    pub(crate) project: String,

    /// Specify the API key of the Cloud Manager.
    #[arg(short = 'k', long)]
    pub(crate) api_key: String,

    /// Specify the API secret of the Cloud Manager.
    #[arg(short = 's', long)]
    pub(crate) api_secret: String,

    /// Specify the resource type for which to fetch the diagnostic data.
    #[arg(short = 't', long, value_enum)]
    pub(crate) resource_type: Resource,

    /// Specify the resource name for which to fetch the diagnostic data.
    ///
    /// For the `cluster` resource type, the value is the name of the
    /// deployment or the cluster id.
    ///
    /// For the `replica-set` resource type, the value is the name of
    /// the replica set in the cluster followed by the shard name.
    /// For example, `test-123abc-shard-0`.
    ///
    /// For the `process` resource type, the value is the name of
    /// the replica set followed by the node name.
    /// For example, `Cluster0-shard-1-node-0`.
    #[arg(short = 'n', long)]
    pub(crate) resource_name: String,

    /// Specify the start timestamp of the diagnostic data.
    #[arg(short = 'r', long)]
    pub(crate) from: Option<DateTime<Utc>>,

    /// Specify the end timestamp of the diagnostic data.
    #[arg(short = 'o', long)]
    pub(crate) to: Option<DateTime<Utc>>,

    /// Specify the path where the diagnostic data will be stored.
    #[arg(short = 'f', long, value_parser(parse_path))]
    pub(crate) path: Option<PathBuf>,

    /// Specify whether the emails, hostnames, IP addresses, and namespaces
    /// are replaced with random string values.
    #[arg(short = 'c', long, default_value_t = true)]
    pub(crate) redacted: bool,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Resource {
    Cluster,
    ReplicaSet,
    Process,
}

fn parse_path(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Err(format!("The `{}` path does not exist.", path.display()));
    }

    if !path.is_dir() {
        return Err(format!(
            "The `{}` path must point to a directory.",
            path.display()
        ));
    }

    Ok(path)
}

pub(crate) trait PathExt {
    fn or_current_dir(self) -> Result<PathBuf, CliError>;
}

impl PathExt for Option<PathBuf> {
    fn or_current_dir(self) -> Result<PathBuf, CliError> {
        if let Some(path) = self {
            Ok(path)
        } else {
            env::current_dir().map_err(|e| CliError::Path(e.to_string()))
        }
    }
}
