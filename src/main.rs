mod cli;

use std::env;

use clap::Parser;
use cli::Cli;
use cli::Commands;

use mprobe::diagnostics::DiagnosticData;

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

            let dd = DiagnosticData::new(&path);
            println!("{dd:?}");

            let r = dd.scan();
            println!("{r:?}");
        }
    }
}
