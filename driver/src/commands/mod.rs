use clap::Subcommand;

use derive_more::From;

mod execute;
pub use execute::*;

mod new;
pub use new::*;

mod init;
pub use init::*;

mod run;
pub use run::*;

mod build;
pub use build::*;

mod compile;
pub use compile::*;

/// The subcommands of ppl
#[derive(Subcommand, Debug, From)]
pub enum Command {
    /// Create a new ppl project at <path>
    New(New),
    /// Create a new ppl package in an existing directory
    Init(Init),
    /// Run ppl binary package
    Run(Run),
    /// Compile ppl package and all dependencies
    Build(Build),
    /// Compile single ppl file
    Compile(Compile),
}

impl Execute for Command {
    type ReturnType = fs_extra::error::Result<()>;

    /// Execute the command
    fn execute(&self) -> Self::ReturnType {
        match self {
            Command::New(new) => new.execute(),
            Command::Init(init) => init.execute(),
            Command::Run(run) => Ok(run.execute()),
            Command::Build(build) => Ok(build.execute()),
            Command::Compile(compile) => Ok(compile.execute()),
        }
    }
}
