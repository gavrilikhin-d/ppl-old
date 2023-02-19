use std::path::{Path, PathBuf};

use clap::Parser;

use super::Execute;

/// Command to create a new ppl package in an existing directory
#[derive(Parser, Debug)]
pub struct Init {
    /// The path to initialize the project at
    #[arg(value_name = "path", default_value = ".")]
    pub path: PathBuf,
}

impl Execute for Init {
    type ReturnType = Result<(), fs_extra::error::Error>;

    /// Create a new ppl package in an existing directory
    fn execute(&self) -> Self::ReturnType {
        let options = fs_extra::dir::CopyOptions::default();
        fs_extra::dir::copy("template", &self.path, &options)?;
        Ok(())
    }
}
