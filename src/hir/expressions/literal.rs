use crate::hir::{Typed, Type};
use crate::syntax::Ranged;

/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { span: std::ops::Range<usize>, value: rug::Integer },
}

impl Ranged for Literal {
	/// Get range of literal
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Literal::None { offset } =>
				*offset..*offset + 4,
			Literal::Integer { span, .. } =>
				span.clone(),
		}
	}
}

impl Typed for Literal {
	/// Get type of literal
	fn ty(&self) -> Type {
		match self {
			Literal::None { .. } => Type::None,
			Literal::Integer { .. } => Type::Integer,
		}
	}
}