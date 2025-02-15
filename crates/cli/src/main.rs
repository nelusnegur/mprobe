mod cli;
mod error;
mod fetch;

use std::env;

use clap::Parser;
use mprobe::diagnostics::DiagnosticData;
use mprobe::diagnostics::MetricsFilter;
use mprobe::vis::layout::VisLayout;

use crate::cli::Cli;
use crate::cli::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // TODO: Validate start_timestamp < end_timestamp
        Commands::View {
            path,
            output_path,
            node: hostname,
            start_timestamp,
            end_timestamp,
        } => {
            let output_path = if let Some(out) = output_path {
                out
            } else {
                env::current_dir().expect("Could not get current working directory.")
            };

            println!(
                "mprobe scans diagnostic data from: `{}` and generates a visual representation in: `{}`",
                path.display(),
                output_path.display()
            );

            let filter = MetricsFilter::new(hostname, start_timestamp, end_timestamp);
            let diagnostic_data = DiagnosticData::filter(&path, filter).expect("valid path");

            let vis =
                VisLayout::init(&output_path).expect("initializing data vis directory failed");
            vis.generate_report(diagnostic_data)
                .expect("generating vis report failed");
        }
        Commands::Fetch(fetch_args) => todo!(),
    }
}
