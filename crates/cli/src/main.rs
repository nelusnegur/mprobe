mod cli;

use std::env;

use clap::Parser;
use mprobe::diagnostics::filter::MetricsFilter;
use mprobe::diagnostics::DiagnosticData;
use mprobe::vis::layout::VisLayout;

use crate::cli::Cli;
use crate::cli::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Visualize { path, output_path, node: hostname, start_timestamp, end_timestamp } => {
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

            // let mut builder = DiagnosticDataBuilder::new(&path);
            // builder.host(String::from("4014e34491b5"));
            // builder.host(hostname);

            // let diagnostic_data = builder.build().expect("valid path");
            let filter = MetricsFilter::new(hostname, start_timestamp, end_timestamp);
            let diagnostic_data = DiagnosticData::filter(&path, filter).expect("valid path");

            let vis =
                VisLayout::init(&output_path).expect("initializing data vis directory failed");
            vis.generate_report(diagnostic_data)
                .expect("generating vis report failed");
        }
    }
}
