use crate::ast::Statement;

use super::{PrecedenceGroups, Token, error::{ParseError, LexerError}, StringWithOffset};

/// Context for parsing
pub struct Context<Lexer: super::Lexer> {
	/// Lexer to use for parsing
	pub lexer: Lexer,
	/// Currently active precedence groups for operators
	pub precedence_groups: PrecedenceGroups
}

impl<Lexer: super::Lexer> Context<Lexer> {
	/// Consume end of line (newline or eof)
	pub fn consume_eol(&mut self) -> Result<Option<StringWithOffset>, LexerError> {
		if self.lexer.peek().is_some() {
			return Ok(Some(self.lexer.consume(Token::Newline)?))
		}
		Ok(None)
	}

	/// Parse block of statements
	pub fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
		let indentation = self.lexer.indentation() + 1;

		self.lexer.consume(Token::Newline)?;
		self.lexer.skip_indentation();

		let mut stmts = Vec::new();
		while self.lexer.indentation() == indentation {
			stmts.push(Statement::parse(self)?);
			self.lexer.skip_indentation();
		}
		Ok(stmts)
	}
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
