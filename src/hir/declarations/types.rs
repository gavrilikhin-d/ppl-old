use crate::{syntax::StringWithOffset, named::Named};

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeDeclaration {
	/// Type's name
	pub name: StringWithOffset,
}

impl Named for TypeDeclaration {
	/// Get name of type
	fn name(&self) -> &str {
		&self.name
	}
}