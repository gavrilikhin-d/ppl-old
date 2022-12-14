use std::collections::HashMap;

use crate::semantics::hir::{VariableDeclaration, TypeDeclaration, Statement};

/// Module with PPL code
#[derive(Debug, PartialEq, Clone)]
pub struct Module {
	/// Variables declared in this module
	pub variables: HashMap<String, VariableDeclaration>,

	/// Types declared in this module
	pub types: HashMap<String, TypeDeclaration>,

	/// Statements in this module
	pub statements: Vec<Statement>
}

impl Module {
	/// Create an empty module
	pub fn new() -> Self {
		Self {
			variables: HashMap::new(),
			types: HashMap::new(),
			statements: vec![]
		}
	}
}