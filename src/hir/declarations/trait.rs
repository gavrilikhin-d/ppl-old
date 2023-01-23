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

impl Named for TraitDeclaration {
	fn name(&self) -> &str {
		&self.name
	}
}
