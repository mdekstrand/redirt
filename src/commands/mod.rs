use clap::Subcommand;
use enum_dispatch::*;

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
