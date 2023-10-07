/// Trait for items that may be generic
pub trait Generic {
    /// Is this a generic item?
    fn is_generic(&self) -> bool;
}
