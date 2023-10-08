use clap::{Args, Subcommand};
use enum_dispatch::*;

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
}

#[derive(Args, Debug)]
struct TraverseFlags {
    /// Follow symbolic links when traversing and copying
    #[arg(short = 'L', long = "follow")]
    follow_symlinks: bool,
}
