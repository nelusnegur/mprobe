mod cli;
mod error;
mod fetch;
mod view;

use clap::Parser;

use crate::cli::Cli;
use crate::cli::Commands;
use crate::error::CliError;
use crate::fetch::fetch;
use crate::view::view;

fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::View(args) => Ok(view(args)?),
        Commands::Fetch(args) => Ok(fetch(args)?),
    }
}
