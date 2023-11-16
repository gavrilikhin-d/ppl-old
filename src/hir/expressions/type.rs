use crate::hir::{Generic, Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// HIR for type reference
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TypeReference {
    /// Range of type reference
    pub span: std::ops::Range<usize>,
    /// Referenced type
    pub referenced_type: Type,
    /// Type of the reference itself
    pub type_for_type: Type,
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
