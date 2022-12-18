use crate::mutability::Mutable;
use crate::syntax::Ranged;
use crate::hir::{Type, Typed, VariableDeclaration};
use std::sync::Arc;

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableReference {
	/// Range of variable reference
	pub span: std::ops::Range<usize>,
	/// Referenced variable name
	pub variable: Arc<VariableDeclaration>,
}

impl Mutable for VariableReference {
	/// Check if referenced variable is mutable
	fn is_mutable(&self) -> bool {
		self.variable.is_mutable()
	}
}

impl Ranged for VariableReference {
	/// Get range of variable reference
	fn range(&self) -> std::ops::Range<usize> {
		self.span.clone()
	}
}

impl Typed for VariableReference {
	/// Get type of variable reference
	fn ty(&self) -> Type {
		self.variable.ty()
	}
}