use std::sync::Arc;

use crate::{named::Named, syntax::StringWithOffset};

use super::FunctionDeclaration;

/// Declaration of a trait
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TraitDeclaration {
    /// Trait's name
    pub name: StringWithOffset,
	/// Associated functions
	pub functions: Vec<Arc<FunctionDeclaration>>
}

impl TraitDeclaration {
	/// Iterate over all functions with `n` name parts
	pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = Arc<FunctionDeclaration>> + '_ {
		self.functions.iter().filter(move |f| f.name_parts.len() == n).cloned()
	}
}

impl Named for TraitDeclaration {
	fn name(&self) -> &str {
		&self.name
	}
}
