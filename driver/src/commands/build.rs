use clap::Parser;

use super::Execute;

/// Command to compile ppl package and all dependencies
#[derive(Parser, Debug)]
pub struct Build {}

impl Execute for Build {
    type ReturnType = ();

    /// Compile ppl package and all dependencies
    fn execute(&self) -> Self::ReturnType {}
}
