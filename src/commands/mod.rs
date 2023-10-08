use clap::Subcommand;
use enum_dispatch::*;

mod list;

#[derive(Subcommand, Debug)]
#[enum_dispatch(Comamnd)]
pub enum DirCommands {
    List(list::ListCmd),
}

/// Interface for RDT commands.
#[enum_dispatch]
pub trait Command {
    fn run(&self) -> anyhow::Result<()>;
}
