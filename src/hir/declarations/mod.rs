mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

mod r#trait;
pub use r#trait::*;

use derive_more::{From, TryInto};

use std::{borrow::Cow, sync::Arc};

use derive_more::Display;

use crate::named::Named;

/// Any PPL declaration
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto)]
pub enum Declaration {
    Variable(Arc<VariableDeclaration>),
    Type(Arc<TypeDeclaration>),
    Function(Function),
    Trait(Arc<TraitDeclaration>),
}

impl Named for Declaration {
    fn name(&self) -> Cow<'_, str> {
        match self {
            Declaration::Variable(decl) => decl.name(),
            Declaration::Type(decl) => decl.name(),
            Declaration::Function(decl) => decl.name(),
            Declaration::Trait(decl) => decl.name(),
        }
    }
}
