use std::fmt::Display;

use derive_more::From;
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[error("expected '{expected}'")]
pub struct Expected {
    /// What was expected
    pub expected: String,
    /// Where the error occurred
    #[label("{expected}")]
    pub at: usize,
}

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[error("expected rule's name")]
pub struct ExpectedRuleName {
    /// Where rule's name was expected
    #[label("here")]
    pub at: usize,
}

#[derive(Debug, Error, Diagnostic, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[error("rule's name doesn't start with a capital letter")]
pub struct RuleNameNotCapitalized {
    /// Offset of the first letter
    #[label("not a capital letter")]
    pub at: usize,
}

/// Helper macro to create error enumeration
macro_rules! error_enum {
	($($name:ident),*) => {
		/// All errors that can occur during parsing
		#[derive(Debug, Error, Diagnostic, PartialEq, Eq, Serialize, Deserialize, Clone, From)]
		pub enum Error {
			$(
				#[error(transparent)]
				#[diagnostic(transparent)]
				$name($name)
			),*
		}
	};
}

error_enum!(
    Expected,
    ExpectedRuleName,
    RuleNameNotCapitalized,
    CustomError
);

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
