use std::sync::Arc;

use super::Generic;

/// Specialized item
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Specialized<G: Generic> {
    /// Generic item to specialize
    pub generic: G,
    /// Specialized item that replaces generic
    pub specialized: G,
}

impl<G: Generic> Specialized<G> {
    /// Is this only partially specialized?
    pub fn is_partially_specialized(&self) -> bool {
        self.is_generic()
    }
}

impl<G: Generic> Generic for Specialized<G> {
    fn is_generic(&self) -> bool {
        self.specialized.is_generic()
    }
}

/// Trait to specialize generic items
pub trait Specialize<S> {
    /// Specialize generic item
    fn specialize_with(self, specialized: S) -> Self;
}

impl<S, T: Specialize<S> + Clone> Specialize<S> for Arc<T> {
    fn specialize_with(self, specialized: S) -> Self {
        Arc::new((*self).clone().specialize_with(specialized))
    }
}
