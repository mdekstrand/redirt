use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use log::*;

mod commands;
pub mod walk;

use commands::{Command, DirCommands};

/// Recursive directory tool.
#[derive(Parser, Debug)]
struct RDTCLI {
    /// Suppress informational output
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,
    /// Increase logging verbosity (can be repeated)
    #[arg(short='v', long="verbose", action=clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: DirCommands,
}

impl RDTCLI {
    fn init_logging(&self) -> Result<()> {
        let mut verbose: usize = 2;
        if self.verbose > 0 {
            verbose += self.verbose as usize;
        } else if self.quiet {
            verbose -= 1;
        }

        stderrlog::new()
            .module(module_path!())
            .verbosity(verbose)
            .init()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let cli = RDTCLI::parse();
    cli.init_logging()?;
    debug!("starting rdt");
    cli.command.run()?;
    Ok(())
}
