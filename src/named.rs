use std::borrow::Cow;

/// Trait for named objects
pub trait Named {
    /// Returns the name of the item.
    fn name(&self) -> Cow<'_, str>;
}
