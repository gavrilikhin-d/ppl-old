use miette::Diagnostic;
use serde::Serialize;
use thiserror::Error;

use crate::IntoParseTreeNode;

/// Trait for errors that support cloning
pub trait Error: Diagnostic + erased_serde::Serialize + Send + Sync + 'static {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static>;
}
impl<E: Error> IntoParseTreeNode for E {}
erased_serde::serialize_trait_object!(Error);

#[derive(Debug, Error, Diagnostic, Serialize, PartialEq, Eq, Clone)]
#[error("expected '{expected}'")]
pub struct Expected {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    #[label("{expected}")]
    pub at: usize,
}
impl Error for Expected {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Error, Diagnostic, Serialize, PartialEq, Eq, Clone)]
#[error("expected typename")]
pub struct ExpectedTypename {
    /// Where typename was expected
    #[label("here")]
    pub at: usize,
}
impl Error for ExpectedTypename {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Error, Diagnostic, Serialize, PartialEq, Eq, Clone)]
#[error("typename doesn't start with a capital letter")]
pub struct TypenameNotCapitalized {
    /// Offset of the first letter
    #[label("not a capital letter")]
    pub at: usize,
}
impl Error for TypenameNotCapitalized {
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
