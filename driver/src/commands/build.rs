use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};
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
pub struct Build {
    /// The directory where all compiler outputs will be placed.
    /// Relative paths are resolved relative to the package's root directory.
    #[arg(long, value_name = "dir", default_value = "target")]
    pub target_dir: PathBuf,
    /// The profile to use for compiling ppl package and its dependencies.
    #[arg(long, value_name = "profile", value_enum, default_value = "debug")]
    pub profile: Profile,
}

impl Default for Build {
    fn default() -> Self {
        Self {
            target_dir: PathBuf::from("target"),
            profile: Profile::Debug,
        }
    }
}

/// The profile to use for compiling the package and its dependencies.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Profile {
    /// Build with debug info, without optimizations
    Debug,
    /// Build with optimizations, without debug info
    Release,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Profile::Debug => write!(f, "debug"),
            Profile::Release => write!(f, "release"),
        }
    }
}

impl Execute for Build {
    type ReturnType = std::io::Result<()>;

    /// Compile ppl package and all dependencies
    fn execute(&self) -> Self::ReturnType {
        let config = Config::get()?;
        let target_dir = if self.target_dir.is_absolute() {
            self.target_dir.clone()
        } else {
            config.dir.join(&self.target_dir)
        };
        std::fs::create_dir_all(target_dir.join(self.profile.to_string()))?;
        Ok(())
    }
}
