use enum_dispatch::enum_dispatch;

/// The mutability of a binding
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Mutability {
    /// The binding is mutable
    Mutable,
    /// The binding is immutable
    Immutable,
}

/// Trait for objects that may be checked for mutability
#[enum_dispatch]
pub trait Mutable {
    /// Is this binding mutable?
    fn is_mutable(&self) -> bool {
        !self.is_immutable()
    }

    /// Is this binding immutable?
    fn is_immutable(&self) -> bool {
        !self.is_mutable()
    }

    /// Get the mutability of this binding
    fn mutability(&self) -> Mutability {
        if self.is_mutable() {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }
}

impl Mutable for Mutability {
    fn is_mutable(&self) -> bool {
        matches!(self, Mutability::Mutable)
    }
}
