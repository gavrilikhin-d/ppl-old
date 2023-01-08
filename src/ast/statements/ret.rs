extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::StartsHere;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};

/// AST for assignment
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Return {
    /// Returned value
    pub value: Option<Expression>,
}

impl StartsHere for Return {
	/// Check that return may start at current lexer position
	fn starts_here(lexer: &mut impl Lexer) -> bool {
		lexer.peek() == Some(Token::Return)
	}
}

impl Parse for Return {
	type Err = ParseError;

	/// Parse return using lexer
	fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Return)?;

		let value = if Expression::starts_here(lexer) {
			Some(Expression::parse(lexer)?)
		} else {
			None
		};

		Ok(Return { value })
	}
}