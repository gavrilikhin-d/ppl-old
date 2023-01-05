use crate::hir::{Type, Typed};
use crate::syntax::Ranged;

/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    /// None literal
    None { offset: usize },
    /// Any precision decimal integer literal
    Integer {
        span: std::ops::Range<usize>,
        value: rug::Integer,
    },
    /// String literal
    String {
        span: std::ops::Range<usize>,
        value: String,
    },
}

impl Ranged for Literal {
    /// Get range of literal
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Literal::None { offset } => *offset..*offset + 4,
            Literal::Integer { span, .. } => span.clone(),
            Literal::String { span, .. } => span.clone(),
        }
    }
}

impl Typed for Literal {
    /// Get type of literal
    fn ty(&self) -> Type {
        match self {
            Literal::None { .. } => Type::None,
            Literal::Integer { .. } => Type::Integer,
            Literal::String { .. } => Type::String,
        }
    }
}
