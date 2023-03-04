use std::path::PathBuf;

use crate::Config;

/// Configuration file was not found
#[derive(thiserror::Error, Debug)]
#[error("'{}' not found in '{dir}' or parent directories", Config::NAME)]
pub struct NotFound {
    /// The directory in which the search started
    pub dir: PathBuf,
}
