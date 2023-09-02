use std::path::PathBuf;

use clap::Parser;

use super::Execute;

/// Command to compile single ppl file
#[derive(Parser, Debug)]
pub struct Compile {
    /// File to compile
    #[arg(value_name = "file")]
    pub file: PathBuf,
    /// Directory where compiler output will be placed.
    #[arg(long, value_name = "dir", default_value = ".")]
    pub output_dir: PathBuf,
}

impl Execute for Compile {
    type Output = ();

    /// Compile single ppl file
    fn execute(&self) -> Self::Output {
        let output = self
            .output_dir
            .join(self.file.file_stem().unwrap())
            .with_extension("ll");
        std::fs::File::create(output).unwrap();
    }
}
