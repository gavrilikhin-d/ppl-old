use std::collections::HashMap;

use crate::semantics::hir::{VariableDeclaration, TypeDeclaration, Statement};
use crate::syntax::ast;

use super::ASTLowering;

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

	/// Get builtin module
	///
	/// # Example
	/// ```
	/// use ppl::semantics::Module;
	///
	/// let module = Module::builtin();
	/// ```
	pub fn builtin() -> Self {
		let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/ppl.ppl");

		let content =
			std::fs::read_to_string(path)
				.expect(format!("Failed to read {}", path).as_str());

		let ast =
			content.parse::<ast::Module>()
				.expect("Errors while parsing builtin module");

		ast.lower_to_hir()
			.expect("Errors while lowering builtin module to hir")
	}
}