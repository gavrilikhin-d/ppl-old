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

/// Errors that can occur during [`Config::get`]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// IO error
    #[error(transparent)]
    IOError(#[from] fs_extra::error::Error),
    /// Error while building package
    #[error(transparent)]
    BuildError(#[from] build::Error),
}

impl Execute for Command {
    type ReturnType = Result<(), Error>;

    /// Execute the command
    fn execute(&self) -> Self::ReturnType {
        match self {
            Command::New(new) => new.execute()?,
            Command::Init(init) => init.execute()?,
            Command::Run(run) => run.execute()?,
            Command::Build(build) => build.execute()?,
            Command::Compile(compile) => compile.execute(),
        };
        Ok(())
    }
}
