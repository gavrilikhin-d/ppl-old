mod compile;
mod new;

use super::Command;

/// Trait for executing commands
pub trait Execute {
    /// The output of the command execution
    type Output = ();

    /// Execute the command
    fn execute(&self) -> Self::Output;
}

impl Execute for Command {
    type Output = miette::Result<()>;

    fn execute(&self) -> Self::Output {
        match self {
            Command::Compile(compile) => compile.execute(),
            Command::New(new) => new.execute(),
        }
    }
}
