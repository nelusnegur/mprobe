mod cli;
mod aggregate;

use std::env;

use clap::Parser;
use mprobe::diagnostics::DiagnosticData;
use mprobe::vis::layout::VisLayout;

use crate::aggregate::AggregateMetricsIter;
use crate::cli::Cli;
use crate::cli::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Visualize { path, output_path } => {
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

            let diagnostic_data = DiagnosticData::new(&path).expect("valid path");
            println!("{diagnostic_data:?}");

            // for metrics in diagnostic_data {
            //     let metrics_chunk = metrics.unwrap();
            //
            //     for m in metrics_chunk.metrics {
            //         println!("{}", m.name)
            //     }
            // }

            let aggregator = AggregateMetricsIter::new(diagnostic_data.into_iter());
            // for item in aggregator {
            //     println!("{:?}", item);
            // }

            let vis = VisLayout::init(&output_path).expect("initializing data vis directory failed");
            vis.generate_report().expect("generating vis report failed");
        }
    }
}
