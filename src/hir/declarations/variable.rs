use std::borrow::Cow;
use std::fmt::Display;

use crate::hir::{Expression, Generic, Type, Typed};
use crate::mutability::{Mutability, Mutable};
use crate::named::Named;
use crate::syntax::StringWithOffset;

/// Declaration of a variable
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableDeclaration {
    /// Variable's name
    pub name: StringWithOffset,
    /// Initializer for variable
    pub initializer: Expression,

    /// Mutability of variable
    pub mutability: Mutability,
}

impl Display for VariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "let {}{} = {}",
            if self.mutability == Mutability::Mutable {
                "mut "
            } else {
                ""
            },
            self.name,
            self.initializer
        )
    }
}

impl Named for VariableDeclaration {
    /// Get name of variable
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Mutable for VariableDeclaration {
    /// Is variable declared as mutable?
    fn is_mutable(&self) -> bool {
        self.mutability.is_mutable()
    }
}

impl Typed for VariableDeclaration {
    /// Get type of variable
    fn ty(&self) -> Type {
        self.initializer.ty()
    }
}

impl Generic for VariableDeclaration {
    fn is_generic(&self) -> bool {
        self.initializer.is_generic()
    }
}
