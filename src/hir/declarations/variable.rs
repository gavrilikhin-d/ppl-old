use crate::mutability::{Mutable, Mutability};
use crate::named::Named;
use crate::syntax::StringWithOffset;
use crate::hir::{Type, Typed, Expression};

/// Declaration of a variable
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableDeclaration {
	/// Variable's name
	pub name: StringWithOffset,
	/// Initializer for variable
	pub initializer: Expression,

	/// Mutability of variable
	pub mutability: Mutability,
}

impl Named for VariableDeclaration {
	/// Get name of variable
	fn name(&self) -> &str {
		&self.name
	}
}

impl Mutable for VariableDeclaration {
	/// Is variable declared as mutable?
	fn is_mutable(&self) -> bool {
		self.mutability.is_mutable()
	}
}

impl Typed for VariableDeclaration {
	/// Get type of variable
	fn ty(&self) -> Type {
		self.initializer.ty()
	}
}