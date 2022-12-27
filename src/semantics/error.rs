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

/// Diagnostic for unknown annotations
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("unknown annotation '@{name}'")]
#[diagnostic(code(semantics::unknown_annotation))]
pub struct UnknownAnnotation {
	/// Name of unknown annotation
	pub name: String,

	/// Span of name
	#[label("here")]
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
#[error("expected {expected} type, got {got}")]
#[diagnostic(code(semantics::type_mismatch))]
pub struct TypeMismatch {
	/// Expected type
	pub expected: Type,
	/// Real type
	pub got: Type,

	/// Span of `from` type
	#[label("this has {expected} type")]
	pub expected_span: SourceSpan,

	/// Span of `to` type
	#[label("this has {got} type")]
	pub got_span: SourceSpan
}

/// Diagnostic for mismatched parameter-argument types
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("parameter {expected} type, got {got}")]
#[diagnostic(code(semantics::argument_type_mismatch))]
pub struct ArgumentTypeMismatch {
	/// Expected type
	pub expected: Type,
	/// Real type
	pub got: Type,

	/// Span of `from` type
	#[label("parameter has {expected} type")]
	pub expected_span: SourceSpan,

	/// Span of `to` type
	#[label("argument has {got} type")]
	pub got_span: SourceSpan
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

/// Diagnostic for unresolved function call
#[derive(Error, Debug, Clone, PartialEq)]
#[error("candidate is not viable")]
pub struct CandidateNotViable {
	/// Expected name of function
	pub reason: Error,
}

impl Diagnostic for CandidateNotViable {
	fn severity(&self) -> Option<miette::Severity> {
		Some(miette::Severity::Advice)
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		self.reason.labels()
	}
}

/// Diagnostic for unresolved function call
#[derive(Error, Debug, Clone, PartialEq)]
#[error("no function '{name}'")]
pub struct NoFunction {
	/// Expected name of function
	pub name: String,

	/// Span of whole function call
	pub at: SourceSpan,

	/// Types of arguments
	pub arguments: Vec<(Type, SourceSpan)>,

	/// Reasons, why candidates failed
	pub candidates: Vec<CandidateNotViable>
}

impl Diagnostic for NoFunction {
	fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
		Some(Box::new("semantics::no_function"))
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
		if self.arguments.is_empty() {
			Some(Box::new(std::iter::once(
				miette::LabeledSpan::new_with_span(
					Some("can't resolve this function call".to_string()), self.at
				)
			)))
		}
		else {
			Some(Box::new(self.arguments
				.iter()
				.map(
					|(t, s)| miette::LabeledSpan::new_with_span(
						Some(format!("<:{}>", t)), *s
					)
				)))
		}
	}

	fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
		if self.candidates.is_empty() {
			None
		}
		else {
			Some(Box::new(self.candidates.iter().map(|c| c as &dyn Diagnostic)))
		}
	}
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
	TypeMismatch(#[from] TypeMismatch),
	#[error(transparent)]
	#[diagnostic(transparent)]
	ArgumentTypeMismatch(#[from] ArgumentTypeMismatch),
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnknownType(#[from] UnknownType),
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnknownAnnotation(#[from] UnknownAnnotation),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoUnaryOperator(#[from] NoUnaryOperator),
	#[error(transparent)]
	#[diagnostic(transparent)]
	NoFunction(#[from] NoFunction),
}