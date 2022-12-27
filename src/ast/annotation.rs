extern crate ast_derive;

use ast_derive::AST;


use crate::syntax::{Token, Lexer, Parse, StringWithOffset, error::ParseError, StartsHere};

use super::Expression;

/// Annotations for statements
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Annotation {
	/// Annotation name
	pub name: StringWithOffset,
	/// Arguments of annotation
	pub args: Vec<Expression>
}

impl StartsHere for Annotation {
	/// Check if annotation 100% starts at current position
	fn starts_here(lexer: &mut Lexer) -> bool {
		lexer.try_match(Token::At).is_ok()
	}
}

impl Parse for Annotation {
	type Err = ParseError;

	/// Parse annotation using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::At)?;

		let name = lexer.consume(Token::Id)?;
		let mut args = Vec::new();
		if lexer.consume(Token::LParen).is_ok() {
			while lexer.peek() != Some(Token::RParen)
			{
				args.push(Expression::parse(lexer)?);
				if lexer.peek() != Some(Token::Colon) { break; }

				lexer.consume(Token::Comma)?;
			}
			lexer.consume(Token::RParen)?;
		}

		Ok(
			Annotation { name, args }
		)
	}
}