extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, Token};

use super::Expression;

/// AST for tuple
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Tuple {
	/// Offset of '('
	pub lparen: usize,
	/// Expression in parentheses
	pub expressions: Vec<Expression>,
	/// Offset of ')'
	pub rparen: usize,
}

impl Ranged for Tuple {
	fn start(&self) -> usize {
		self.lparen
	}

	fn end(&self) -> usize {
		self.rparen + 1
	}
}

impl StartsHere for Tuple {
	/// Check that tuple may start at current lexer position
	fn starts_here(lexer: &mut impl Lexer) -> bool {
		lexer.peek() == Some(Token::LParen)
	}
}

impl Parse for Tuple {
	type Err = ParseError;

	fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
		let lparen = lexer.consume(Token::LParen)?.start();

		let mut expressions = Vec::new();
		loop {
			expressions.push(Expression::parse(lexer)?);

			if lexer.peek().map_or(true, |t| t != Token::Comma) {
				break;
			}

			lexer.consume(Token::Comma)?;

			if lexer.peek().map_or(true, |t| t == Token::RParen) {
				break;
			}
		}

		let rparen = lexer.consume(Token::RParen)?.start();

		Ok(Tuple {
			lparen,
			expressions,
			rparen,
		})
	}
}
