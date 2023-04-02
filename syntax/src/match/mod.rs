mod group;
pub use group::*;

mod pattern;
pub use pattern::*;

mod rule;
pub use rule::*;

mod repeat;
pub use repeat::*;

/// Trait for matched patterns
pub trait Match<'source> {
    /// Check if match has no error nodes
    fn is_ok(&self) -> bool;

    /// Check if match has error nodes
    fn has_error(&self) -> bool {
        !self.is_ok()
    }

    /// Get matched tokens
    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_>;

    /// Interpret the match as a single token
    fn as_token(&self) -> &'source str {
        self.tokens().next().unwrap()
    }

    /// Iterate over submatches
    fn submatches(&self) -> Box<dyn Iterator<Item = &PatternMatch<'source>> + '_> {
        Box::new(std::iter::empty())
    }
}
