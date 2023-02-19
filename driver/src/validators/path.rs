use std::path::PathBuf;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

/// Diagnostic for a wrong path
#[derive(Error, Debug, Diagnostic)]
#[error("path doesn't exist")]
#[diagnostic(code(driver::wrong_path))]
pub struct WrongPath {
    /// Span to the path
    #[label("path doesn't exist")]
    at: SourceSpan,
}

/// Error's that may appear in [`existing_path`] function
#[derive(Error, Diagnostic, Debug)]
pub enum ExistingPathError {
    #[error(transparent)]
    #[diagnostic(code(driver::io_error))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    WrongPath(#[from] WrongPath),
}

/// Check that path exists
pub fn exists(path: &str) -> Result<PathBuf, ExistingPathError> {
    let path = PathBuf::from(path);
    if path.try_exists()? {
        return Ok(path);
    }
    Err(WrongPath { at: (0..0).into() }.into())
}

/// Diagnostic for a path, that's not a directory
#[derive(Error, Debug, Diagnostic)]
#[error("expected path to directory")]
#[diagnostic(code(driver::expected_directory))]
pub struct ExpectedDirectory {
    /// Span to the path
    #[label("isn't a directory")]
    at: SourceSpan,
}

/// Error's that may appear in [`existing_dir`] function
#[derive(Error, Diagnostic, Debug)]
pub enum ExistingDirError {
    #[error(transparent)]
    ExistingPathError(#[from] ExistingPathError),

    #[error(transparent)]
    ExpectedDirectory(#[from] ExpectedDirectory),
}

/// Check that path exists and is a directory
pub fn is_dir(path: &str) -> Result<PathBuf, ExistingDirError> {
    let p = exists(path)?;
    if p.is_dir() {
        return Ok(p);
    }
    Err(ExpectedDirectory { at: (0, 0).into() }.into())
}
