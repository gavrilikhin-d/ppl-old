use std::sync::Arc;

use super::Generic;

/// Specialized item
#[derive(PartialEq, Eq, Clone)]
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
pub trait Specialize<S>: Sized {
    type Specialized = Self;

    // TODO: change to &mut self
    /// Specialize generic item
    fn specialize_with(self, specialized: S) -> Self::Specialized;
}

impl<S, T, U: Specialize<S, Specialized = T> + Clone> Specialize<S> for Arc<U> {
    type Specialized = Arc<T>;

    fn specialize_with(self, specialized: S) -> Self::Specialized {
        Arc::new((*self).clone().specialize_with(specialized))
    }
}
