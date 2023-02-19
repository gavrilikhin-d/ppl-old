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
    /// Create a new ppl package in an existing directory
    fn execute(&self) {
        println!("Initializing project at {:?}", self.path);
    }
}
