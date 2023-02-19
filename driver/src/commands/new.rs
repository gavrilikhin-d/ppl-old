use std::path::PathBuf;

use clap::Parser;

use super::{Execute, Init};

#[derive(Parser, Debug)]
pub struct New {
    /// The path to create the project at
    #[arg(value_name = "path")]
    pub path: PathBuf,
}

impl Execute for New {
    fn execute(&self) {
        println!("Creating new project at {:?}", self.path);
        Init {
            path: self.path.clone(),
        }
        .execute();
    }
}
