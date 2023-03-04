use std::{fmt::Display, path::PathBuf};

use clap::{Parser, ValueEnum};

use crate::{config, Config};

use super::{Compile, Execute};

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

/// Errors that can occur during [`Build`] execution
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// IO error
    #[error(transparent)]
    IOError(#[from] fs_extra::error::Error),
    /// Error while reading configuration
    #[error(transparent)]
    ConfigurationError(#[from] config::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err.into())
    }
}

impl Execute for Build {
    type ReturnType = Result<(), Error>;

    /// Compile ppl package and all dependencies
    fn execute(&self) -> Self::ReturnType {
        let config = Config::get()?;
        let target_dir = if self.target_dir.is_absolute() {
            self.target_dir.clone()
        } else {
            config.dir.join(&self.target_dir)
        };
        let output_dir = target_dir.join(self.profile.to_string());
        std::fs::create_dir_all(&output_dir)?;
        let source_dir = config.dir.join("src");
        let files = fs_extra::dir::get_dir_content(source_dir)?.files;
        let sources = files
            .iter()
            .map(|entry| PathBuf::from(entry))
            .filter(|path| path.extension().map(|str| str.to_str()).flatten() == Some("ppl"));
        let compile_requests = sources.map(|src| Compile {
            file: src,
            output_dir: output_dir.clone(),
        });
        compile_requests.for_each(|r| r.execute());
        Ok(())
    }
}
