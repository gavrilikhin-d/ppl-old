use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

use super::Type;

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable '{name}' is not defined")]
#[diagnostic(code(evaluator::undefined_variable))]
pub struct UndefinedVariable {
	/// Name of undefined variable
	pub name: String,

	/// Span of name
	#[label("reference to undefined variable")]
	pub at: SourceSpan
}

/// Diagnostic for assignment to immutable
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("assignment to immutable")]
#[diagnostic(code(evaluator::assignment_to_immutable))]
pub struct AssignmentToImmutable {
	/// Span of immutable thing
	#[label("this value is immutable")]
	pub at: SourceSpan,
}

/// Diagnostic for not convertible types
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't convert {from:?} to {to:?}")]
#[diagnostic(code(evaluator::no_conversion))]
pub struct NoConversion {
	/// Type, that must be converted
	pub from: Type,
	/// Target type
	pub to: Type,

	/// Span of `from` type
	#[label("this has {from:?} type")]
	pub from_span: SourceSpan,

	/// Span of `to` type
	#[label("this has {to:?} type")]
	pub to_span: SourceSpan
}

/// Possible lexer errors
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
pub enum Error {
	#[error(transparent)]
	#[diagnostic(transparent)]
	UndefinedVariable(#[from] UndefinedVariable),
	#[error(transparent)]
	#[diagnostic(transparent)]
	AssignmentToImmutable(#[from] AssignmentToImmutable),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoConversion(#[from] NoConversion),
}