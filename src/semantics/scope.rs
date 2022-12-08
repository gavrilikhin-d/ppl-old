use super::hir::VariableDeclaration;

/// Lexical scope
pub struct Scope {
	/// Parent scope
	parent: Option<Box<Scope>>,

	/// Variables declared in the scope
	variables: std::collections::HashMap<String, VariableDeclaration>,
}