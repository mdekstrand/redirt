use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use happylog::clap::LogOpts;
use log::*;

mod commands;
pub mod walk;

use commands::{Command, DirCommands};

/// Recursive directory tool.
#[derive(Parser, Debug)]
struct RDTCLI {
    #[command(flatten)]
    logging: LogOpts,

    #[command(subcommand)]
    command: DirCommands,
}

fn main() -> Result<()> {
    let cli = RDTCLI::parse();
    cli.logging.init()?;
    debug!("starting rdt");
    cli.command.run();
    Ok(())
}
