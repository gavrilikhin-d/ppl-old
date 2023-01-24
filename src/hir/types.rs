use std::{fmt::Display, sync::Arc};

use crate::{mutability::Mutable, named::Named};

use super::{TypeDeclaration, Module, TraitDeclaration};
use derive_more::From;

/// PPL's Function type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionType {
	/// Parameters
	pub parameters: Vec<Type>,
	/// Return type
	pub return_type: Box<Type>,
	/// Cached name of function type
	name: String
}

impl FunctionType {
	/// Build new function type
	pub fn build() -> FunctionTypeBuilder {
		FunctionTypeBuilder::new()
	}
}

impl Named for FunctionType {
	fn name(&self) -> &str {
		&self.name
	}
}

/// Builder for FunctionType
pub struct FunctionTypeBuilder {
	/// Parameters
	pub parameters: Vec<Type>,
}

impl FunctionTypeBuilder {
	/// Create new builder for function type
	pub fn new() -> Self {
		Self {
			parameters: Vec::new()
		}
	}

	/// Set parameter to function type
	pub fn with_parameters(mut self, parameters: Vec<Type>) -> Self {
		self.parameters = parameters;
		self
	}

	/// Set return type to function type and build function
	pub fn with_return_type(self, return_type: Type) -> FunctionType {
		let name = self.build_name(&return_type);
		FunctionType {
			parameters: self.parameters,
			return_type: Box::new(return_type),
			name
		}
	}

	/// Build name of function type
	fn build_name(&self, return_type: &Type) -> String {
		let mut name = String::new();
		name.push_str("(");
		for (i, parameter) in self.parameters.iter().enumerate() {
			if i != 0 {
				name.push_str(", ");
			}
			name.push_str(parameter.name());
		}
		name.push_str(&format!(") -> {}", return_type.name()));
		name
	}
}

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Type {
    /// User defined type
    Class(Arc<TypeDeclaration>),
	/// User defined trait
	Trait(Arc<TraitDeclaration>),
	/// Self type, used in traits
	SelfType,
    /// Function type
    Function(FunctionType),
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

	/// Get builtin "Bool" type
	pub fn bool() -> Type {
		Type::get_builtin("Bool")
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
		write!(f, "{}", self.name())
    }
}

impl Named for Type {
	fn name(&self) -> &str {
		match self {
			Type::Class(class) => class.name(),
			Type::Trait(tr) => tr.name(),
			Type::SelfType => "Self",
			Type::Function(f) => f.name()
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
