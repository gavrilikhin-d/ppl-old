use miette::{Diagnostic, SourceOffset};
use thiserror::Error;

use crate::IntoParseTreeNode;

/// Trait for errors that support cloning
pub trait Error: Diagnostic + Send + Sync + 'static {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static>;
}
impl<E: Error> IntoParseTreeNode for E {}

#[derive(Debug, Error, Diagnostic, PartialEq, Eq, Clone)]
#[error("expected '{expected}'")]
pub struct Expected {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    #[label("{expected}")]
    pub at: SourceOffset,
}
impl Error for Expected {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected one of {variants:?}")]
pub struct ExpectedOneOf<'i> {
    /// What was expected
    pub variants: Vec<String>,
    /// Where the error occurred
    pub at: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("expected matching {expected:?} to match {to_match:?}")]
pub struct ExpectedMatching<'i> {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    pub at: &'i str,
    /// What was expected to match
    pub to_match: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("referencing unknown rule {name:?}")]
pub struct UnknownRuleReference<'i> {
    /// Unknown rule name
    pub name: &'i str,
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("type error")]
pub struct TypeError {}
