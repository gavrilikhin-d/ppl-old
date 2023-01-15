use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::hir::{Type, CallKind};

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable '{name}' is not defined")]
#[diagnostic(code(semantics::undefined_variable))]
pub struct UndefinedVariable {
    /// Name of undefined variable
    pub name: String,

    /// Span of name
    #[label("reference to undefined variable")]
    pub at: SourceSpan,
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
    pub at: SourceSpan,
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
    pub at: SourceSpan,
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
    pub got_span: SourceSpan,
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
    pub got_span: SourceSpan,
}

/// Diagnostic for mismatched condition type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("condition must have \"Bool\" type, got \"{got}\"")]
#[diagnostic(code(semantics::condition_type_mismatch))]
pub struct ConditionTypeMismatch {
    /// Real type
    pub got: Type,

    /// Span of got type
    #[label("this expression has {got} type")]
    pub at: SourceSpan,
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
pub struct NoFunction {
	/// Kind of function call, that failed to bind to function
	pub kind: CallKind,

    /// Expected name of function
    pub name: String,

    /// Span of whole function call
    pub at: SourceSpan,

    /// Types of arguments
    pub arguments: Vec<(Type, SourceSpan)>,

    /// Reasons, why candidates failed
    pub candidates: Vec<CandidateNotViable>,
}

impl Display for NoFunction {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.kind {
			CallKind::Call =>
				write!(f, "no function \"{}\"", self.name),
			CallKind::Operation =>
				write!(f, "no operator \"{}\"", self.name),
		}
	}
}

impl Diagnostic for NoFunction {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("semantics::no_function"))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        if self.arguments.is_empty() {
            Some(Box::new(std::iter::once(
                miette::LabeledSpan::new_with_span(
                    Some("can't resolve this function call".to_string()),
                    self.at,
                ),
            )))
        } else {
			let mut labels = Vec::new();
			if self.kind == CallKind::Operation {
				labels.push(miette::LabeledSpan::new_with_span(
					Some("for this operator".to_string()),
					self.at,
				))
			}
			labels.extend(
				self.arguments.iter().map(
					|(t, s)| miette::LabeledSpan::new_with_span(
						Some(format!("<:{}>", t)), *s
					)
				)
			);
            Some(Box::new(labels.into_iter()))
        }
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        if self.candidates.is_empty() {
            None
        } else {
            Some(Box::new(
                self.candidates.iter().map(|c| c as &dyn Diagnostic),
            ))
        }
    }
}

/// Diagnostic for return statement outside of function
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("return outside of function")]
#[diagnostic(code(semantics::return_outside_function))]
pub struct ReturnOutsideFunction {
    /// Span of return statement
    #[label("this return is outside of function")]
    pub at: SourceSpan,
}

/// Diagnostic for missing return value
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("missing return value with \"{ty}\" type")]
#[diagnostic(code(semantics::missing_return_value))]
pub struct MissingReturnValue {
	/// Type of return value
	pub ty: Type,
    /// Point, where return value is expected
    #[label("here")]
    pub at: SourceSpan,
}

/// Diagnostic for mismatch of return type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("return type mismatch: got \"{got}\", expected \"{expected}\"")]
#[diagnostic(code(semantics::return_type_mismatch))]
pub struct ReturnTypeMismatch {
	/// Type of return value
	pub got: Type,
    /// Span of returned value
    #[label("this has \"{got}\" type")]
    pub got_span: SourceSpan,

	/// Expected type of return value
	pub expected: Type,
}

/// Helper macro to create error enumeration
macro_rules! error_enum {
	($($name:ident),*) => {
		/// Possible semantics errors
		#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
		pub enum Error {
			$(
				#[error(transparent)]
				#[diagnostic(transparent)]
				$name(#[from] $name)
			),*
		}
	};
}

error_enum!(
	UndefinedVariable,
	AssignmentToImmutable,
    TypeMismatch,
    ArgumentTypeMismatch,
    ConditionTypeMismatch,
    UnknownType,
    UnknownAnnotation,
    NoFunction,
    ReturnOutsideFunction,
    MissingReturnValue,
    ReturnTypeMismatch
);
