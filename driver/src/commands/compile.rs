use std::path::PathBuf;

use clap::Parser;

use super::Execute;

/// Command to compile single ppl file
#[derive(Parser, Debug)]
pub struct Compile {
    /// File to compile
    #[arg(value_name = "file")]
    pub file: PathBuf,
    /// Name of the output file
    #[arg(short, long, value_name = "target")]
    pub output: Option<PathBuf>,
}

impl Execute for Compile {
    type ReturnType = ();

    /// Compile single ppl file
    fn execute(&self) -> Self::ReturnType {}
}
