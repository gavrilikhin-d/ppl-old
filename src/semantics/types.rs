use std::fmt::Display;

use super::hir::TypeDeclaration;
use derive_more::From;

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Type {
	/// None type
	None,
	/// Arbitrary integer type
	Integer,
	/// User defined type
	Class(TypeDeclaration),
	/// Function type
	Function { parameters: Vec<Type>, return_type: Box<Type> },
}

impl Display for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::None => write!(f, "None"),
			Type::Integer => write!(f, "Integer"),
			Type::Class(class) => write!(f, "{}", class.name.value),
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

/// Trait for values with a type
pub trait Typed {
	/// Get type of value
	fn get_type(&self) -> Type;
}