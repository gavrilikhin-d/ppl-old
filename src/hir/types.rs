use std::{fmt::Display, sync::Arc};

use crate::mutability::Mutable;

use super::TypeDeclaration;
use derive_more::From;

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Type {
	/// None type
	None,
	/// Arbitrary integer type
	Integer,
	/// User defined type
	Class(Arc<TypeDeclaration>),
	/// Function type
	Function { parameters: Vec<Type>, return_type: Box<Type> },
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::None => write!(f, "None"),
			Type::Integer => write!(f, "Integer"),
			Type::Class(class) => write!(f, "{}", class.name),
			Type::Function { parameters, return_type } => {
				write!(f, "(")?;
				for (i, parameter) in parameters.iter().enumerate() {
					if i != 0 {
						write!(f, ", ")?;
					}
					write!(f, "{}", parameter)?;
				}
				write!(f, ") -> {}", return_type)
			}
		}
	}
}

impl Mutable for Type {
	fn is_mutable(&self) -> bool {
		false
	}
}

/// Trait for values with a type
pub trait Typed {
	/// Get type of value
	fn ty(&self) -> Type;
}