use std::path::PathBuf;

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
        let files = std::fs::read_dir("template")
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        let options = fs_extra::dir::CopyOptions::default();
        fs_extra::copy_items(&files, &self.path, &options)?;
        std::process::Command::new("git")
            .arg("init")
            .arg(&self.path)
            .output()?;
        Ok(())
    }
}
