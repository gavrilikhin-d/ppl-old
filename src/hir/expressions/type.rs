use std::fmt::Display;

use derive_visitor::DriveMut;

use crate::hir::{Generic, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// HIR for type reference
#[derive(Debug, PartialEq, Eq, Hash, Clone, DriveMut)]
pub struct TypeReference {
    /// Range of type reference
    #[drive(skip)]
    pub span: std::ops::Range<usize>,
    /// Referenced type
    #[drive(skip)]
    pub referenced_type: Type,
    /// Type of the reference itself
    #[drive(skip)]
    pub type_for_type: Type,
}

impl Display for TypeReference {
    /// Display type reference
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.referenced_type)
    }
}

impl Generic for TypeReference {
    fn is_generic(&self) -> bool {
        self.referenced_type.is_generic()
    }
}

impl Mutable for TypeReference {
    /// Type reference is always immutable
    fn is_immutable(&self) -> bool {
        true
    }
}

impl Ranged for TypeReference {
    /// Get range of type reference
    fn range(&self) -> std::ops::Range<usize> {
        self.span.clone()
    }
}

impl Typed for TypeReference {
    /// Get type of variable reference
    fn ty(&self) -> Type {
        self.type_for_type.clone()
    }
}
