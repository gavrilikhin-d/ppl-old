use std::fmt::{self, Debug, Display, Formatter};

use crate::hir::{Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// AST for compile time known values
#[derive(PartialEq, Eq, Clone)]
pub enum Literal {
    /// None literal
    None { offset: usize, ty: Type },
    /// Bool literal
    Bool {
        offset: usize,
        value: bool,
        ty: Type,
    },
    /// Any precision decimal integer literal
    Integer {
        span: std::ops::Range<usize>,
        value: rug::Integer,
        ty: Type,
    },
    /// Any precision decimal rational literal
    Rational {
        span: std::ops::Range<usize>,
        value: rug::Rational,
        ty: Type,
    },
    /// String literal
    String {
        span: std::ops::Range<usize>,
        value: String,
        ty: Type,
    },
}

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Literal::None { .. } => write!(f, "none"),
            Literal::Bool { value, .. } => write!(f, "{}", value),
            Literal::Integer { value, .. } => write!(f, "{}", value),
            Literal::Rational { value, .. } => write!(f, "{}", value),
            Literal::String { value, .. } => write!(f, "{:?}", value),
        }
    }
}

impl Ranged for Literal {
    /// Get range of literal
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Literal::None { offset, .. } => *offset..*offset + "none".len(),
            Literal::Bool { offset, value, .. } => *offset..*offset + format!("{}", value).len(),
            Literal::Integer { span, .. } => span.clone(),
            Literal::Rational { span, .. } => span.clone(),
            Literal::String { span, .. } => span.clone(),
        }
    }
}

impl Typed for Literal {
    /// Get type of literal
    fn ty(&self) -> Type {
        match self {
            Literal::None { ty, .. } => ty,
            Literal::Bool { ty, .. } => ty,
            Literal::Integer { ty, .. } => ty,
            Literal::Rational { ty, .. } => ty,
            Literal::String { ty, .. } => ty,
        }
        .clone()
    }
}

impl Mutable for Literal {
    /// Literal is always immutable
    fn is_immutable(&self) -> bool {
        true
    }
}
