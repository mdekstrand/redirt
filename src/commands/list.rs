//! directory listing command

use std::path::PathBuf;

use clap::Args;
use log::*;

use crate::walk::WalkBuilder;

use super::{Command, TraverseFlags};

/// List a directory.
#[derive(Debug, Args)]
#[command(name = "list")]
pub struct ListCmd {
    #[command(flatten)]
    traverse: TraverseFlags,

    /// The directory to list.
    #[arg(name = "DIR")]
    dir: PathBuf,
}

impl Command for ListCmd {
    fn run(&self) -> anyhow::Result<()> {
        info!("listing direectory {:?}", self.dir);
        let mut walk = WalkBuilder::for_directory(&self.dir);
        walk = walk.follow_symlinks(self.traverse.follow_symlinks);
        walk = walk.include_hidden(self.traverse.include_hidden);
        let walk = walk.walk();
        let mut n = 0;
        for entry in walk {
            let entry = entry?;
            n += 1;
            println!("{}", entry.path().display());
        }
        info!("{}: walked {} entries", self.dir.display(), n);
        Ok(())
    }
}
