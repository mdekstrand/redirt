//! directory listing command

use std::path::PathBuf;

use clap::Args;
use log::*;

use crate::{
    diff::{DiffEntry, TreeDiffBuilder},
    walk::WalkBuilder,
};

use super::{Command, TraverseFlags};

/// Compare two directories.
#[derive(Debug, Args)]
#[command(name = "compare")]
pub struct DiffCmd {
    #[command(flatten)]
    traverse: TraverseFlags,

    /// Include unchanged files in output.
    #[arg(short = 'u', long = "unchaged")]
    list_unchanged: bool,

    /// The source directory to compare.
    #[arg(name = "SRC")]
    source: PathBuf,

    /// The target directory to compare.
    #[arg(name = "TGT")]
    target: PathBuf,
}

impl Command for DiffCmd {
    fn run(&self) -> anyhow::Result<()> {
        info!("source directory {:?}", self.source);
        info!("target directory {:?}", self.target);
        let mut diff = TreeDiffBuilder::new(&self.source, &self.target);
        diff.follow_symlinks(self.traverse.follow_symlinks);
        diff.include_hidden(self.traverse.include_hidden);

        let diff = diff.run();

        let mut n = 0;
        for entry in diff {
            let entry = entry?;
            n += 1;
            match entry {
                DiffEntry::Present { src, tgt } => {
                    if self.list_unchanged {
                        println!("P: {}", src.path().display());
                    }
                }
                DiffEntry::Added { src } => {
                    println!("A: {}", src.path().display());
                }
                DiffEntry::Removed { tgt } => {
                    println!("R: {}", tgt.path().display());
                }
                DiffEntry::Modified { src, tgt, .. } => {
                    println!("M: {}", src.path().display());
                }
            }
        }

        info!("found {} diff entries", n);
        Ok(())
    }
}
