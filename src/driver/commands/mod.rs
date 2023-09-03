use clap::Subcommand;

use derive_more::From;

mod compile;
pub use compile::*;

mod execute;
pub use execute::*;

use std::error::Error;

/// The subcommands of ppl
#[derive(Subcommand, Debug, From)]
pub enum Command {
    /// Compile single ppl file
    Compile(Compile),
}

impl Execute for Command {
    type Output = Result<(), Box<dyn Error>>;

    fn execute(&self) -> Self::Output {
        match self {
            Command::Compile(compile) => compile.execute(),
        }
    }
}
