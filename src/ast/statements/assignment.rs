extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{Token, Lexer, Parse, error::ParseError};

/// AST for assignment
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Assignment {
	/// Target to assign to
	pub target: Expression,
	/// Expression to assign
	pub value: Expression,
}

impl Parse for Assignment {
	type Err = ParseError;

	/// Parse assignment using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let target = Expression::parse(lexer)?;

		lexer.consume(Token::Assign)?;

		let value = Expression::parse(lexer)?;

		Ok(Assignment { target, value })
	}
}