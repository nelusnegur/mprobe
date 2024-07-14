mod cli;
mod aggregate;

use std::env;

use clap::Parser;
use mprobe::diagnostics::DiagnosticData;
use mprobe::vis::layout::chart::Chart;
use mprobe::vis::layout::section::Section;
use mprobe::vis::layout::view::View;
use mprobe::vis::layout::ElementKind;
use mprobe::vis::render::output::OutputFile;
use mprobe::vis::render::Render;

use crate::aggregate::AggregateMetricsIter;
use crate::cli::Cli;
use crate::cli::Commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { path, output_path } => {
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
            for item in aggregator {
                println!("{:?}", item);
            }

            let view = View::new()
                .insert(ElementKind::Section(Section::new()))
                .insert(ElementKind::Chart(Chart::new()))
                .insert(ElementKind::Chart(Chart::new()));

            let mut output = OutputFile::new(&output_path).unwrap();
            view.render(&mut output).unwrap();
        }
    }
}
