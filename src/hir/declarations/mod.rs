mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

use derive_more::{From, TryInto};

use std::sync::Arc;

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Declaration {
	Variable(Arc<VariableDeclaration>),
	Type(Arc<TypeDeclaration>),
	Function(Arc<FunctionDeclaration>),
}