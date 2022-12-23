use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

use crate::hir::Type;

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable '{name}' is not defined")]
#[diagnostic(code(semantics::undefined_variable))]
pub struct UndefinedVariable {
	/// Name of undefined variable
	pub name: String,

	/// Span of name
	#[label("reference to undefined variable")]
	pub at: SourceSpan
}

/// Diagnostic for unknown type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("unknown type '{name}'")]
#[diagnostic(code(semantics::unknown_type))]
pub struct UnknownType {
	/// Name of unknown type
	pub name: String,

	/// Span of name
	#[label("reference to unknown type")]
	pub at: SourceSpan
}

/// Diagnostic for assignment to immutable
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("assignment to immutable")]
#[diagnostic(code(semantics::assignment_to_immutable))]
pub struct AssignmentToImmutable {
	/// Span of immutable thing
	#[label("this value is immutable")]
	pub at: SourceSpan,
}

/// Diagnostic for not convertible types
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't convert {from:?} to {to:?}")]
#[diagnostic(code(semantics::no_conversion))]
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

/// Diagnostic for unresolved unary operator
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("no unary operator '{name}'")]
#[diagnostic(code(semantics::no_unary_operator))]
pub struct NoUnaryOperator {
	/// Expected name of unary operator
	pub name: String,

	/// Type of operand
	pub operand_type: Type,

	/// Span of operator
	#[label("can't resolve this unary operator")]
	pub operator_span: SourceSpan,

	/// Span of operator
	#[label("<:{operand_type}>")]
	pub operand_span: SourceSpan,
}

/// Possible semantics errors
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
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnknownType(#[from] UnknownType),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoUnaryOperator(#[from] NoUnaryOperator),
}