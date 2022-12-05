use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable '{name}' is not defined")]
#[diagnostic(code(evaluator::undefined_variable))]
pub struct UndefinedVariable {
	/// Name of undefined variable
	pub name: String,

	/// Span of name
	#[label]
	pub at: SourceSpan
}