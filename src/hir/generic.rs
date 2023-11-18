use std::borrow::Cow;

/// Trait for items that may be generic
pub trait Generic {
    /// Is this a generic item?
    fn is_generic(&self) -> bool;
}

/// Trait to get name without generic parameters
pub trait Basename {
    /// Get name without generic parameters
    fn basename(&self) -> Cow<'_, str>;
}
