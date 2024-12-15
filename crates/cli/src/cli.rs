use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub(crate) struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Visualize the diagnostic data and generate a visual representation of it.
    Visualize {
        /// Specify the path from where to read the diagnostic data.
        /// The path must exist and it must point to a directory.
        #[clap(short, long, parse(try_from_os_str = parse_path))]
        path: PathBuf,

        /// Specify the path where the generated output will be created.
        /// If the output path is not specified then the current working
        /// directory is used.
        #[clap(short, long, parse(try_from_os_str = parse_path))]
        output_path: Option<PathBuf>,

        /// Filter metrics by the host name.
        #[clap(short, long)]
        node: Option<String>,

        /// Specify the start timestamp of the metrics.
        #[clap(short, long)]
        start_timestamp: Option<DateTime<Utc>>,

        /// Specify the end timestamp of the metrics.
        #[clap(short, long)]
        end_timestamp: Option<DateTime<Utc>>,
    },
}

fn parse_path(path: &OsStr) -> Result<PathBuf, String> {
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
