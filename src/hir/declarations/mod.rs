mod function;
use derive_visitor::DriveMut;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

mod r#trait;
pub use r#trait::*;

use derive_more::{From, TryInto};

use std::borrow::Cow;

use derive_more::Display;

use crate::{named::Named, syntax::Ranged};

/// Any PPL declaration
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto, DriveMut)]
pub enum Declaration {
    Variable(Variable),
    #[drive(skip)]
    Type(Class),
    Function(Function),
    Trait(Trait),
}

impl Named for Declaration {
    fn name(&self) -> Cow<'_, str> {
        match self {
            Declaration::Variable(decl) => decl.name(),
            Declaration::Type(decl) => decl.read().unwrap().name().to_string().into(),
            Declaration::Function(decl) => decl.name(),
            Declaration::Trait(decl) => decl.name(),
        }
    }
}

impl Ranged for Declaration {
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Declaration::Variable(decl) => decl.range(),
            Declaration::Type(decl) => decl.range(),
            Declaration::Function(decl) => decl.range(),
            Declaration::Trait(decl) => decl.range(),
        }
    }
}
