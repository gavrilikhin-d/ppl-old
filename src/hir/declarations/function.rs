use std::fmt::Display;

use derive_more::From;

use crate::named::Named;
use crate::syntax::StringWithOffset;
use crate::hir::{Type, Typed};

/// Declaration of a function parameter
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
	/// Type's name
	pub name: StringWithOffset,
	/// Type of parameter
	pub ty: Type
}

impl Named for Parameter {
	/// Get name of parameter
	fn name(&self) -> &str {
		&self.name
	}
}

impl Typed for Parameter {
	/// Get type of parameter
	fn ty(&self) -> Type {
		self.ty.clone()
	}
}

/// Part of a function name
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum FunctionNamePart {
	Text(StringWithOffset),
	Parameter(Parameter),
}

impl Display for FunctionNamePart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FunctionNamePart::Text(text) => write!(f, "{}", text),
			FunctionNamePart::Parameter(parameter) =>
				write!(f, "<{}: {}>", parameter.name, parameter.ty),
		}
	}
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDeclaration {
	/// Type's name
	pub name_parts: Vec<FunctionNamePart>,
	/// Type of returned value
	pub return_type: Type,

	/// Cached name of function
	name: String,
}

/// Builder for a function declaration
pub struct FunctionDeclarationBuilder {
	/// Type's name
	name_parts: Vec<FunctionNamePart>,
}

impl FunctionDeclarationBuilder {
	/// Create a new builder for a function declaration
	pub fn new() -> FunctionDeclarationBuilder {
		FunctionDeclarationBuilder {
			name_parts: Vec::new(),
		}
	}

	/// Set name parts of the function
	pub fn with_name(mut self, name_parts: Vec<FunctionNamePart>)
	-> FunctionDeclarationBuilder {
		self.name_parts = name_parts;
		self
	}

	/// Set the return type of the function and return the declaration
	pub fn with_return_type(self, return_type: Type) -> FunctionDeclaration {
		let mut name = String::new();

		for (i, part) in self.name_parts.iter().enumerate() {
			if i > 0 {
				name.push_str(" ");
			}
			name.push_str(format!("{}", part).as_str());
		}

		FunctionDeclaration {
			name_parts: self.name_parts,
			return_type,
			name,
		}
	}
}

impl FunctionDeclaration {
	/// Create a new builder for a function declaration
	pub fn build() -> FunctionDeclarationBuilder {
		FunctionDeclarationBuilder::new()
	}
}

impl Named for FunctionDeclaration {
	/// Get name of function
	fn name(&self) -> &str {
		&self.name
	}
}

impl Typed for FunctionDeclaration {
	fn ty(&self) -> Type {
		Type::Function {
			return_type: Box::new(self.return_type.clone()),
			parameters:
				self.name_parts.iter().filter_map(
					|part| match part {
						FunctionNamePart::Parameter(parameter) =>
							Some(parameter.ty()),
						_ => None
					}
				).collect()
		}
	}
}
