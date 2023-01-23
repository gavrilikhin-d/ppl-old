mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

mod r#trait;
pub use r#trait::*;

use derive_more::{From, TryInto};

use std::sync::Arc;

use crate::named::Named;

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Declaration {
    Variable(Arc<VariableDeclaration>),
    Type(Arc<TypeDeclaration>),
    Function(Arc<FunctionDeclaration>),
	Trait(Arc<TraitDeclaration>),
}

impl Named for Declaration {
    fn name(&self) -> &str {
        match self {
            Declaration::Variable(decl) => decl.name(),
            Declaration::Type(decl) => decl.name(),
            Declaration::Function(decl) => decl.name(),
			Declaration::Trait(decl) => decl.name(),
        }
    }
}
