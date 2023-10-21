use std::error::Error;

use miette::Diagnostic;
use thiserror::Error;

/// Array of errors
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("")]
pub struct ErrVec<E: Diagnostic + Error> {
    /// Errors in array
    #[related]
    pub errors: Vec<E>,
}

impl<E: Diagnostic + Error> From<Vec<E>> for ErrVec<E> {
    fn from(errors: Vec<E>) -> Self {
        Self { errors }
    }
}
