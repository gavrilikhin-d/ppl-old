use crate::{named::Named, syntax::StringWithOffset};

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeDeclaration {
    /// Type's name
    pub name: StringWithOffset,
	/// Is this type from builtin module?
	pub is_builtin: bool,
}

impl TypeDeclaration {
	/// Is this a builtin type?
	pub fn is_builtin(&self) -> bool {
		self.is_builtin
	}

	/// Is this a builtin "None" type?
	pub fn is_none(&self) -> bool {
		self.is_builtin && self.name == "None"
	}

	/// Is this a builtin "Bool" type?
	pub fn is_bool(&self) -> bool {
		self.is_builtin && self.name == "Bool"
	}

	/// Is this a builtin "Integer" type?
	pub fn is_integer(&self) -> bool {
		self.is_builtin && self.name == "Integer"
	}

	/// Is this a builtin "String" type?
	pub fn is_string(&self) -> bool {
		self.is_builtin && self.name == "String"
	}
}

impl Named for TypeDeclaration {
    /// Get name of type
    fn name(&self) -> &str {
        &self.name
    }
}
