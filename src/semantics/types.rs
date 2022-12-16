use super::hir::TypeDeclaration;

/// Type of values
#[derive(Debug, PartialEq, Eq, Clone)]
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

/// Trait for values with a type
pub trait Typed {
	/// Get type of value
	fn get_type(&self) -> Type;
}