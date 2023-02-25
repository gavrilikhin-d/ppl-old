use clap::Parser;

use super::{Build, Execute};

/// Command to run ppl binary package
#[derive(Parser, Debug)]
pub struct Run {}

impl Execute for Run {
    type ReturnType = <Build as Execute>::ReturnType;

    /// Run ppl binary package
    fn execute(&self) -> Self::ReturnType {
        return Build {}.execute();
    }
}
