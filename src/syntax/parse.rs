use crate::syntax::Lexer;

/// Trait for parsing using lexer
pub trait Parse
where
    Self: Sized,
{
    type Err;

    /// Parse starting from current lexer state
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err>;
}

/// Trait for checking that current lexer state is 100% start of a node
pub trait StartsHere {
    /// Check if current lexer state is 100% start of this node
    fn starts_here(lexer: &mut impl Lexer) -> bool;
}
