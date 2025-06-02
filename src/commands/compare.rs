//! directory listing command

use std::path::PathBuf;

use anstream::println;
use anstyle::{AnsiColor, Color, Style};
use clap::Args;
use log::*;

use crate::diff::{DiffEntry, TreeDiffBuilder};

use super::{Command, TraverseFlags};

const PRESENT_STYLE: Style = Style::new().dimmed();
const REMOVED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
const ADDED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
const MODIFIED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));

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
                        println!(
                            "  {}{}{}",
                            PRESENT_STYLE.render(),
                            src.path().display(),
                            PRESENT_STYLE.render_reset()
                        );
                    }
                }
                DiffEntry::Added { src } => {
                    println!(
                        "+ {}{}{}",
                        ADDED_STYLE.render(),
                        src.path().display(),
                        ADDED_STYLE.render_reset()
                    );
                }
                DiffEntry::Removed { tgt } => {
                    println!(
                        "- {}{}{}",
                        REMOVED_STYLE.render(),
                        tgt.path().display(),
                        REMOVED_STYLE.render_reset()
                    );
                }
                DiffEntry::Modified { src, tgt: _, .. } => {
                    println!(
                        "x {}{}{}",
                        MODIFIED_STYLE.render(),
                        src.path().display(),
                        MODIFIED_STYLE.render_reset()
                    );
                }
            }
        }

        info!("found {} diff entries", n);
        Ok(())
    }
}
