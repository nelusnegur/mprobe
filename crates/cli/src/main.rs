mod cli;

use std::env;

use clap::Parser;
use mprobe::diagnostics::DiagnosticData;
use mprobe::vis::layout::VisLayout;

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

            let vis = VisLayout::init(&output_path).expect("initializing data vis directory failed");
            vis.generate_report(diagnostic_data).expect("generating vis report failed");
        }
    }
}
