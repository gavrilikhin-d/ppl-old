mod not_found;
pub use not_found::*;

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
