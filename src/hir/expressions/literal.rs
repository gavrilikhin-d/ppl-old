use crate::hir::{Type, Typed};
use crate::syntax::Ranged;

/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
    /// None literal
    None { offset: usize, ty: Type },
	/// Bool literal
	Bool { offset: usize, value: bool, ty: Type },
    /// Any precision decimal integer literal
    Integer {
        span: std::ops::Range<usize>,
        value: rug::Integer,
		ty: Type,
    },
    /// String literal
    String {
        span: std::ops::Range<usize>,
        value: String,
		ty: Type
    },
}

impl Ranged for Literal {
    /// Get range of literal
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Literal::None { offset, .. } => *offset..*offset + "none".len(),
			Literal::Bool { offset, value, .. } =>
				if *value {
					*offset..*offset + "true".len()
				}
				else {
					*offset..*offset + "false".len()
				}
            Literal::Integer { span, .. } => span.clone(),
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
            Literal::String { ty, .. } => ty,
        }.clone()
    }
}
