use std::path::PathBuf;

use clap::Parser;

use super::Execute;

/// Build configuration
#[derive(Debug)]
struct Config {
    /// Path to the config file
    pub path: PathBuf,
}

impl Config {
    /// Recursively search for a config file
    pub fn find() -> std::io::Result<Config> {
        std::fs::read_dir(".")?
            .filter_map(|entry| entry.ok())
            .find(|entry| entry.file_name() == "build.config")
            .map(|entry| Config { path: entry.path() })
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, "build.config not found")
            })
    }
}

/// Command to compile ppl package and all dependencies
#[derive(Parser, Debug)]
pub struct Build {}

impl Execute for Build {
    type ReturnType = std::io::Result<()>;

    /// Compile ppl package and all dependencies
    fn execute(&self) -> Self::ReturnType {
        let config = Config::find()?;
        println!("{:?}", config);
        Ok(())
    }
}
