use std::sync::Arc;

use crate::{named::Named, syntax::StringWithOffset, hir::Type};

/// Member of type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member {
    /// Member's name
    pub name: StringWithOffset,
	/// Member's type
	pub ty: Type,
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeDeclaration {
    /// Type's name
    pub name: StringWithOffset,
	/// Is this type from builtin module?
	pub is_builtin: bool,
	/// Members of type
	pub members: Vec<Arc<Member>>,
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ast;
	use crate::hir::{Type, Member};
	use crate::semantics::ASTLowering;

	#[test]
	fn test_type_without_body() {
		let type_decl =
			"type x"
				.parse::<ast::TypeDeclaration>()
				.unwrap()
				.lower_to_hir()
				.unwrap();

		assert_eq!(
			*type_decl,
			TypeDeclaration {
				name: StringWithOffset::from("x").at(5),
				is_builtin: false,
				members: vec![],
			}
		);
	}

	#[test]
	fn test_type_with_body() {
		let type_decl =
			include_str!("../../../examples/point.ppl")
				.parse::<ast::TypeDeclaration>()
				.unwrap()
				.lower_to_hir()
				.unwrap();

		assert_eq!(
			*type_decl,
			TypeDeclaration {
				name: StringWithOffset::from("Point").at(5),
				is_builtin: false,
				members: vec![
					Arc::new(Member {
						name: StringWithOffset::from("x").at(13),
						ty: Type::integer(),
					}),
					Arc::new(Member {
						name: StringWithOffset::from("y").at(16),
						ty: Type::integer(),
					}),
				],
			}
		);
	}
}