use std::{fmt::Display, sync::Arc};

use crate::mutability::Mutable;

use super::{TypeDeclaration, Module};
use derive_more::From;

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Type {
    /// User defined type
    Class(Arc<TypeDeclaration>),
    /// Function type
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
}

impl Type {
	/// Is this a builtin type?
	pub fn is_builtin(&self) -> bool {
		match self {
			Type::Class(c) => c.is_builtin(),
			_ => false
		}
	}

	/// Is this a builtin "None" type?
	pub fn is_none(&self) -> bool {
		match self {
			Type::Class(c) => c.is_none(),
			_ => false
		}
	}

	/// Is this a builtin "Integer" type?
	pub fn is_integer(&self) -> bool {
		match self {
			Type::Class(c) => c.is_integer(),
			_ => false
		}
	}

	/// Is this a builtin "String" type?
	pub fn is_string(&self) -> bool {
		match self {
			Type::Class(c) => c.is_string(),
			_ => false
		}
	}

	/// Get builtin type by name
	fn get_builtin(name: &str) -> Type {
		Module::builtin().types.get(name).unwrap().value.clone().into()
	}

	/// Get builtin "None" type
	pub fn none() -> Type {
		Type::get_builtin("None")
	}

	/// Get builtin "Integer" type
	pub fn integer() -> Type {
		Type::get_builtin("Integer")
	}

	/// Get builtin "String" type
	pub fn string() -> Type {
		Type::get_builtin("String")
	}
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Class(class) => write!(f, "{}", class.name),
            Type::Function {
                parameters,
                return_type,
            } => {
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
