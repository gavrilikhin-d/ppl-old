use std::path::PathBuf;

use clap::Parser;

use super::Execute;

#[derive(Parser, Debug)]
pub struct Init {
    /// The path to initialize the project at
    #[arg(value_name = "path", default_value = ".")]
    pub path: PathBuf,
}

impl Execute for Init {
    fn execute(&self) {
        println!("Initializing project at {:?}", self.path);
    }
}
