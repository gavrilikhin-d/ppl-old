use crate::hir::{Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// HIR for type reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeReference {
    /// Range of type reference
    pub span: std::ops::Range<usize>,
    /// Referenced type
    pub referenced_type: Type,
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
        unimplemented!("Type for types")
    }
}
