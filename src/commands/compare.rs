//! directory listing command

use std::path::PathBuf;

use clap::Args;
use log::*;

use crate::diff::{DiffEntry, TreeDiffBuilder};

use super::{Command, TraverseFlags};

/// Compare two directories.
#[derive(Debug, Args)]
#[command(name = "compare")]
pub struct DiffCmd {
    #[command(flatten)]
    traverse: TraverseFlags,

    /// Check the content of files if times and sizes are identical.
    #[arg(short = 'C', long = "check-content")]
    check_content: bool,

    /// Include unchanged files in output.
    #[arg(short = 'u', long = "unchanged")]
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
        diff.check_content(self.check_content);

        let diff = diff.run();

        let mut n = 0;
        for entry in diff {
            let entry = entry?;
            n += 1;
            match entry {
                DiffEntry::Present { src, tgt: _ } => {
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
                DiffEntry::Modified { src, tgt: _, .. } => {
                    println!("M: {}", src.path().display());
                }
            }
        }

        info!("found {} diff entries", n);
        Ok(())
    }
}
