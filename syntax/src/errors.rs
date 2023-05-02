use std::fmt::Display;

use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::IntoParseTreeNode;

/// Trait for errors that support cloning
pub trait Error: Diagnostic + erased_serde::Serialize + Send + Sync + 'static {
    fn clone_boxed(&self) -> Box<dyn Diagnostic + Send + Sync + 'static>;
}
impl<E: Error> IntoParseTreeNode for E {}
erased_serde::serialize_trait_object!(Error);

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

/// Severity of the error. Copied from miette to add `Serialize` and `Deserialize` traits.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Severity {
    /// Just some help. Here's how you could be doing it better.
    Advice,
    /// Warning. Please take note.
    Warning,
    /// Critical failure. The program cannot continue.
    Error,
}

impl From<Severity> for miette::Severity {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Advice => miette::Severity::Advice,
            Severity::Warning => miette::Severity::Warning,
            Severity::Error => miette::Severity::Error,
        }
    }
}

type Offset = usize;
type Length = usize;

/// A labeled [`SourceSpan`]. Copied from miette to add `Serialize` and `Deserialize` traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabeledSpan {
    pub label: Option<String>,
    pub span: (Offset, Length),
}

impl From<LabeledSpan> for miette::LabeledSpan {
    fn from(value: LabeledSpan) -> Self {
        miette::LabeledSpan::new_with_span(value.label, value.span)
    }
}

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct CustomError {
    /// Severity of the error
    pub severity: Severity,
    /// Error message
    pub message: String,
    /// Unique diagnostic code
    pub code: Option<String>,
    /// Help message
    pub help: Option<String>,
    /// Labels on the source code
    pub labels: Option<Vec<LabeledSpan>>,
    /// URL to the documentation
    pub url: Option<String>,
}

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Diagnostic for CustomError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.code.as_ref().map(|s| Box::new(s) as Box<dyn Display>)
    }
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.help.as_ref().map(|s| Box::new(s) as Box<dyn Display>)
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.labels.as_ref().map(|labels| {
            Box::new(labels.iter().cloned().map(|l| l.into()))
                as Box<dyn Iterator<Item = miette::LabeledSpan>>
        })
    }
    fn severity(&self) -> Option<miette::Severity> {
        Some(self.severity.into())
    }
    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.url.as_ref().map(|s| Box::new(s) as Box<dyn Display>)
    }
}
