use std::collections::HashMap;

use crate::semantics::hir::{VariableDeclaration, Statement};

/// Module with PPL code
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
	/// Variables declared in this module
	pub variables: HashMap<String, VariableDeclaration>,

	/// Statements in this module
	pub statements: Vec<Statement>
}

impl Module {
	/// Create an empty module
	pub fn new() -> Self {
		Self { variables: HashMap::new(), statements: vec![] }
	}
}