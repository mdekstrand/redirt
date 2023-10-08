use clap::{Args, Subcommand};
use enum_dispatch::*;

use crate::walk::WalkBuilder;

mod compare;
mod list;

/// Interface for RDT commands.
#[enum_dispatch]
pub trait Command {
    fn run(&self) -> anyhow::Result<()>;
}

#[derive(Subcommand, Debug)]
#[enum_dispatch(Command)]
pub enum DirCommands {
    List(list::ListCmd),
    Compare(compare::DiffCmd),
}

#[derive(Args, Debug)]
struct TraverseFlags {
    /// Follow symbolic links when traversing and copying
    #[arg(short = 'L', long = "follow")]
    follow_symlinks: bool,

    /// Include hidden files
    #[arg(short = 'H', long = "hidden")]
    include_hidden: bool,
}

impl TraverseFlags {
    fn apply_settings(&self, walk: &mut WalkBuilder) {
        walk.follow_symlinks(self.follow_symlinks);
        walk.include_hidden(self.include_hidden);
    }
}
