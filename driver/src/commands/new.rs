use std::path::PathBuf;

use clap::Parser;

use super::Execute;

/// Command to create a new ppl project at <path>
#[derive(Parser, Debug)]
pub struct New {
    /// The path to create the project at
    #[arg(value_name = "path")]
    pub path: PathBuf,
}

impl Execute for New {
    type ReturnType = fs_extra::error::Result<()>;

    /// Create a new ppl project at <path>
    fn execute(&self) -> Self::ReturnType {
        let options = fs_extra::dir::CopyOptions::default();
        fs_extra::dir::copy("template", &self.path, &options)?;
        Ok(())
    }
}
