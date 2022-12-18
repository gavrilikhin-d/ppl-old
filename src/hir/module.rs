use std::collections::HashSet;

use std::sync::Arc;
use crate::ast;
use crate::hir::{VariableDeclaration, TypeDeclaration, FunctionDeclaration, Statement};
use crate::named::HashByName;
use crate::semantics::ASTLowering;

/// Module with PPL code
#[derive(Debug, PartialEq, Eq)]
pub struct Module {
	/// Variables, declared in this module
	pub variables: HashSet<HashByName<Arc<VariableDeclaration>>>,

	/// Types, declared in this module
	pub types: HashSet<HashByName<Arc<TypeDeclaration>>>,

	/// Functions, declared in this module
	pub functions: HashSet<HashByName<Arc<FunctionDeclaration>>>,

	/// Statements in this module
	pub statements: Vec<Statement>
}

impl Module {
	/// Create an empty module
	pub fn new() -> Self {
		Self {
			variables: HashSet::new(),
			types: HashSet::new(),
			functions: HashSet::new(),
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