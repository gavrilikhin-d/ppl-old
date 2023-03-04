use std::path::PathBuf;

use serde::Deserialize;

/// Build configuration
#[derive(Debug, Deserialize)]
pub struct Config {
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
pub struct Package {
    /// Name of the package
    pub name: String,
}

/// Errors that can occur during [`Config::get`]
pub mod error {
    use std::path::PathBuf;

    use crate::Config;

    /// Configuration file was not found
    #[derive(thiserror::Error, Debug)]
    #[error("'{}' not found in '{dir}' or parent directories", Config::NAME)]
    pub struct NotFound {
        /// The directory in which the search started
        pub dir: PathBuf,
    }

    /// Errors that can occur during [`Config::get`]
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// IO error
        #[error(transparent)]
        IOError(#[from] std::io::Error),
        /// Configuration file was not found
        #[error(transparent)]
        NotFound(#[from] NotFound),
        /// Configuration is invalid
        #[error(transparent)]
        InvalidConfig(#[from] toml::de::Error),
    }
}

pub use error::Error;

use self::error::NotFound;

impl Config {
    /// Recursively search for a config file and read it
    pub fn get() -> Result<Config, Error> {
        let cwd = std::env::current_dir()?;
        let path = cwd
            .ancestors()
            .map(|path| path.join(Config::NAME))
            .find(|path| path.exists())
            .ok_or_else(|| NotFound { dir: cwd })?;

        let content = std::fs::read_to_string(&path)?;
        let mut config: Config = toml::from_str(&content)?;
        // Unwrapping here is safe, this directory exists
        config.dir = path.parent().unwrap().to_owned();
        Ok(config)
    }
}
