use std::{fmt::Display, sync::Arc};

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use derive_more::From;

use crate::{
    ast::FnKind,
    hir::{TraitData, Type},
    SourceFile,
};

/// Diagnostic for undefined variables
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("variable `{name}` is not defined")]
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
#[error("unknown type `{name}`")]
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
#[error("unknown annotation `@{name}`")]
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

/// Show type in diagnostic
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("{ty}")]
pub struct TypeWithSpan {
    /// Type to show
    pub ty: Type,

    /// Span of expected type
    #[label("this has `{ty}` type")]
    pub at: SourceSpan,

    /// Source code of the module, this type is located at
    #[source_code]
    pub source_file: Option<SourceFile>,
}

impl From<TypeWithSpan> for SourceSpan {
    fn from(value: TypeWithSpan) -> Self {
        value.at
    }
}

/// Diagnostic for not convertible types
#[derive(Error, Debug, Clone, PartialEq)]
#[error("expected `{expected}` type, got `{got}`")]
pub struct TypeMismatch {
    /// Expected type
    pub expected: TypeWithSpan,
    /// Real type
    pub got: TypeWithSpan,
}

impl Diagnostic for TypeMismatch {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("semantics::type_mismatch"))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        Some(Box::new(
            vec![
                &self.expected as &dyn Diagnostic,
                &self.got as &dyn Diagnostic,
            ]
            .into_iter(),
        ))
    }
}

/// Diagnostic for mismatched condition type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("condition must have `Bool` type, got `{got}`")]
#[diagnostic(code(semantics::condition_type_mismatch))]
pub struct ConditionTypeMismatch {
    /// Real type
    pub got: Type,

    /// Span of got type
    #[label("this expression has `{got}` type")]
    pub at: SourceSpan,
}

/// Diagnostic for unresolved unary operator
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("no unary operator `{name}`")]
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
#[derive(Diagnostic, Error, Debug, Clone, PartialEq)]
#[error("candidate is not viable")]
#[diagnostic(severity(Advice))]
pub struct CandidateNotViable {
    #[source]
    #[diagnostic_source]
    pub reason: Error,
}

/// Diagnostic for unresolved function call
#[derive(Error, Debug, Clone, PartialEq)]
pub struct NoFunction {
    /// Kind of function call, that failed to bind to function
    pub kind: FnKind,

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
            FnKind::Function => write!(f, "no function `{}`", self.name),
            FnKind::Operator => write!(f, "no operator `{}`", self.name),
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
            if self.kind == FnKind::Operator {
                labels.push(miette::LabeledSpan::new_with_span(
                    Some("for this operator".to_string()),
                    self.at,
                ))
            }
            labels.extend(
                self.arguments.iter().map(|(t, s)| {
                    miette::LabeledSpan::new_with_span(Some(format!("<:{}>", t)), *s)
                }),
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
#[error("missing return value with `{ty}` type")]
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
#[error("return type mismatch: got `{got}`, expected `{expected}`")]
#[diagnostic(code(semantics::return_type_mismatch))]
pub struct ReturnTypeMismatch {
    /// Type of return value
    pub got: Type,
    /// Span of returned value
    #[label("this has `{got}` type")]
    pub got_span: SourceSpan,

    /// Expected type of return value
    pub expected: Type,
}

/// Diagnostic for recursive implicit return type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't deduce implicit return type of function")]
#[diagnostic(code(semantics::cant_deduce_return_type))]
pub struct CantDeduceReturnType {
    /// Span of function
    #[label("Can't deduce return type of this function")]
    pub at: SourceSpan,
}

/// Diagnostic for types that compiler can't infer
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't deduce type")]
#[diagnostic(code(semantics::cant_deduce_type))]
pub struct CantDeduceType {
    /// Span of function
    #[label("Can't deduce type of this")]
    pub at: SourceSpan,
}

/// Diagnostic for missing members
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("no member `{name}` in `{ty}`")]
#[diagnostic(code(semantics::no_member))]
pub struct NoMember {
    /// Type of base expression
    pub ty: Type,
    /// Span of base expression
    #[label("this has `{ty}` type")]
    pub base_span: SourceSpan,

    /// Span of function
    #[label("no member `{name}` in `{ty}`")]
    pub at: SourceSpan,
    /// Name of member
    pub name: String,
}

/// Diagnostic for multiple initializers for single field
#[derive(Diagnostic, Error, Debug, Clone, PartialEq)]
#[error("field `{name}` initialized multiple times")]
#[diagnostic(code(semantics::multiple_initialization))]
pub struct MultipleInitialization {
    /// Name of the field
    pub name: String,
    /// First initializer span
    #[label("was firstly initialized here")]
    pub first_at: SourceSpan,
    /// Span of the initializers
    #[label(collection, "repeated initialization")]
    pub repeated_at: Vec<SourceSpan>,
}

/// Diagnostic for missing fields in constructor
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("type `{ty}` has missing fields: {fields}")]
#[diagnostic(code(semantics::missing_fields))]
pub struct MissingFields {
    /// Type of constructed class
    pub ty: Type,
    /// Span of constructor name
    #[label("missing fields: {fields}")]
    pub at: SourceSpan,
    /// Names of missing fields
    pub fields: DisplayVec<String>,
}

/// Wrapper around [`Vec`] to display it
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub struct DisplayVec<D: Display>(pub Vec<D>);

impl<D: Display> Display for DisplayVec<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.0
                .iter()
                .map(|p| format!("`{}`", p))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Diagnostic for using constructor with non-class type
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("constructor can be used only with class types")]
#[diagnostic(code(semantics::non_class_constructor))]
pub struct NonClassConstructor {
    #[label("Can't construct this non-class type")]
    pub ty: TypeWithSpan,
}

/// Diagnostic for unimplemented trait
#[derive(Diagnostic, Error, Debug, Clone, PartialEq)]
#[error("`{ty}` doesn't satisfy trait `{tr}` requirements")]
#[diagnostic(code(semantics::not_implemented))]
pub struct NotImplemented {
    /// Type that doesn't satisfy trait requirements
    pub ty: Type,
    /// Trait, that is not implemented
    pub tr: Arc<TraitData>,
    /// Unimplemented functions spans
    #[label(collection, "This required function isn't implemented")]
    pub unimplemented: Vec<SourceSpan>,
}

/// Diagnostic for trying to take mutable reference to immutable data
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("can't take mutable reference to immutable data")]
#[diagnostic(code(semantics::reference_mut_to_immutable))]
pub struct ReferenceMutToImmutable {
    #[label("This value is immutable")]
    pub at: SourceSpan,
}

/// Diagnostic for not convertible types
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
pub enum NotConvertible {
    #[error(transparent)]
    #[diagnostic(transparent)]
    TypeMismatch(#[from] TypeMismatch),
    #[error(transparent)]
    #[diagnostic(transparent)]
    NotImplemented(#[from] NotImplemented),
    #[error(transparent)]
    #[diagnostic(transparent)]
    ReferenceMutToImmutable(#[from] ReferenceMutToImmutable),
}

/// Diagnostic for unresolved import
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("unresolved import of `{name}`")]
#[diagnostic(code(semantics::unresolved_import))]
pub struct UnresolvedImport {
    /// Name of the unresolved item
    pub name: String,
    /// Location of the unresolved item
    #[label("No such item im module")]
    pub at: SourceSpan,
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
    ConditionTypeMismatch,
    UnknownType,
    UnknownAnnotation,
    NoFunction,
    ReturnOutsideFunction,
    MissingReturnValue,
    ReturnTypeMismatch,
    CantDeduceReturnType,
    CantDeduceType,
    NoMember,
    MultipleInitialization,
    MissingFields,
    NonClassConstructor,
    NotImplemented,
    NotConvertible,
    UnresolvedImport
);
