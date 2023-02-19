use clap::Subcommand;

use derive_more::From;

mod execute;
pub use execute::*;

mod new;
pub use new::*;

mod init;
pub use init::*;

/// The subcommands of ppl
#[derive(Subcommand, Debug, From)]
pub enum Command {
    /// Create a new ppl project at <path>
    New(New),
    /// Create a new ppl package in an existing directory
    Init(Init),
}

impl Execute for Command {
    fn execute(&self) -> Self::ReturnType {
        match self {
            Command::New(new) => new.execute(),
            Command::Init(init) => init.execute(),
        }
    }
}
