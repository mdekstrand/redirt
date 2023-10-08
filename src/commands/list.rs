//! directory listing command

use std::path::PathBuf;

use clap::Args;
use log::*;

use crate::walk::WalkBuilder;

use super::Command;

/// List a directory.
#[derive(Debug, Args)]
#[command(name = "list")]
pub struct ListCmd {
    /// The directory to list.
    #[arg(name = "DIR")]
    dir: PathBuf,
}

impl Command for ListCmd {
    fn run(&self) -> anyhow::Result<()> {
        let walk = WalkBuilder::for_directory(&self.dir).walk();
        let mut n = 0;
        for entry in walk {
            n += 1;
            println!("dir");
        }
        info!("{}: walked {} entries", self.dir.display(), n);
        Ok(())
    }
}
