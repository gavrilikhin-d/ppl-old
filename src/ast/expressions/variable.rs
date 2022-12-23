extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, Ranged, StringWithOffset, error::ParseError};

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct VariableReference {
	/// Referenced variable name
	pub name: StringWithOffset
}

impl Parse for VariableReference {
	type Err = ParseError;

	/// Parse variable reference using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		Ok(VariableReference { name: lexer.consume(Token::Id)? })
	}
}

impl Ranged for VariableReference {
	/// Get range of variable reference
	fn range(&self) -> std::ops::Range<usize> {
		self.name.range()
	}
}