//! directory listing command

use std::path::PathBuf;

use clap::Args;
use log::*;

use crate::walk::{walk_fs, WalkOptions};

use super::Command;

/// List a directory.
#[derive(Debug, Args)]
#[command(name = "list")]
pub struct ListCmd {
    #[command(flatten)]
    traverse: WalkOptions,

    /// List directories after their contents
    #[arg(long = "dirs-last")]
    dirs_last: bool,

    /// The directory to list.
    #[arg(name = "DIR")]
    dir: PathBuf,
}

impl Command for ListCmd {
    fn run(&self) -> anyhow::Result<()> {
        info!("listing direectory {:?}", self.dir);
        let walk = walk_fs(&self.dir, &self.traverse)?;
        if self.dirs_last {
            panic!("unsupported option")
        }
        let mut n = 0;
        for entry in walk {
            let entry = entry?;
            n += 1;
            let sfx = if entry.is_directory() { "/" } else { "" };
            println!("{}{}", entry.path().display(), sfx);
        }
        info!("{}: walked {} entries", self.dir.display(), n);
        Ok(())
    }
}
