use crate::hir::{Generic, Specialize, Type, Typed};
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

impl Generic for TypeReference {
    fn is_generic(&self) -> bool {
        self.referenced_type.is_generic()
    }
}

impl Specialize<Type> for TypeReference {
    fn specialize_with(mut self, specialized: Type) -> Self {
        self.referenced_type = self.referenced_type.specialize_with(specialized).into();
        self
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
        unimplemented!("Type for types")
    }
}
