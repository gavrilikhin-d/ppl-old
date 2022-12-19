use crate::syntax::Lexer;

/// Trait for parsing using lexer
pub trait Parse where Self: Sized {
	type Err;

	/// Parse starting from current lexer state
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err>;
}