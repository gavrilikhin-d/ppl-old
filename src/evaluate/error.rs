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
	#[label("reference to undefined variable")]
	pub at: SourceSpan
}

/// Diagnostic for assignment to immutable variable
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("assignment to immutable variable '{name}'")]
#[diagnostic(code(evaluator::assignment_to_immutable))]
pub struct AssignmentToImmutable {
	/// Name of variable
	pub name: String,

	/// Span of name in declaration
	#[label("assigning to this immutable variable")]
	pub referenced_at: SourceSpan,

	/// Span of name in declaration
	#[label("variable declared here as immutable")]
	pub declared_at: SourceSpan
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
}