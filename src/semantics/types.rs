use super::hir::TypeDeclaration;

/// Type of values
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
	/// None type
	None,
	/// Arbitrary integer type
	Integer,
	/// User defined type
	Class(TypeDeclaration)
}

/// Trait for values with a type
pub trait Typed {
	/// Get type of value
	fn get_type(&self) -> Type;
}