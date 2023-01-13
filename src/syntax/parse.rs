use super::PrecedenceGroups;

/// Context for parsing
pub struct Context<Lexer: super::Lexer> {
	/// Lexer to use for parsing
	pub lexer: Lexer,
	/// Currently active precedence groups for operators
	pub precedence_groups: PrecedenceGroups
}

impl <'l, Lexer: super::Lexer> Context<Lexer> {
	/// Create new context with default precedence groups
	pub fn new(lexer: Lexer) -> Self {
		Self {
			lexer,
			precedence_groups: PrecedenceGroups::default()
		}
	}
}

/// Trait for parsing using context.lexer
pub trait Parse
where
    Self: Sized,
{
    type Err;

    /// Parse starting from current context.lexer state
    fn parse(context: &mut Context<impl super::Lexer>)
		-> Result<Self, Self::Err>;
}

/// Trait for checking that current context.lexer state is 100% start of a node
pub trait StartsHere {
    /// Check if current context.lexer state is 100% start of this node
    fn starts_here(context: &mut Context<impl super::Lexer>) -> bool;
}
