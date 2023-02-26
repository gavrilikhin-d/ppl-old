use std::path::PathBuf;

use clap::Parser;
use serde::Deserialize;

use super::Execute;

/// Build configuration
#[derive(Debug, Deserialize)]
struct Config {
    /// Directory of the config file
    #[serde(skip)]
    pub dir: PathBuf,
    /// Package information
    pub package: Package,
}

impl Config {
    /// Name of the config file
    const NAME: &'static str = "build.config";
}

/// Package information
#[derive(Debug, Deserialize)]
struct Package {
    /// Name of the package
    pub name: String,
}

impl Config {
    /// Recursively search for a config file and read it
    pub fn get() -> std::io::Result<Config> {
        let cwd = std::env::current_dir()?;
        let path = cwd
            .ancestors()
            .map(|path| path.join(Config::NAME))
            .find(|path| path.exists())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!(
                        "{} not found in '{}' or parent directories",
                        Config::NAME,
                        cwd.display()
                    ),
                )
            })?;

        let content = std::fs::read_to_string(&path)?;
        let mut config: Config = toml::from_str(&content).unwrap();
        config.dir = path.parent().unwrap().to_owned();
        Ok(config)
    }
}

/// Command to compile ppl package and all dependencies
#[derive(Parser, Debug)]
pub struct Build {}

impl Execute for Build {
    type ReturnType = std::io::Result<()>;

    /// Compile ppl package and all dependencies
    fn execute(&self) -> Self::ReturnType {
        let config = Config::get()?;
        println!("{:?}", config);
        Ok(())
    }
}
