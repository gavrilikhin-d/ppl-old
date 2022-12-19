mod assignment;
pub use assignment::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::ast::{Declaration, Expression};
use crate::syntax::{Token, Lexer, Parse, error::ParseError};

use derive_more::From;

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Statement {
	Declaration(Declaration),
	Expression(Expression),
	Assignment(Assignment),
}

impl Parse for Statement {
	type Err = ParseError;

	/// Parse statement using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(
			&[
				Token::None, Token::Integer, Token::Id,
				Token::Let, Token::Plus, Token::Minus,
				Token::Type, Token::Fn
			]
		);
		if token.is_err() {
			return Err(token.unwrap_err().into())
		}

		let res = match token.unwrap() {
			Token::Let | Token::Type | Token::Fn =>
				Declaration::parse(lexer)
					.map(|decl| Statement::Declaration(decl)),
			Token::None | Token::Integer | Token::Id |
			Token::Plus | Token::Minus => {
				let target = Expression::parse(lexer)?;

				if lexer.consume(Token::Assign).is_err() {
					Ok(target.into())
				}
				else
				{
					Ok(
						Assignment {
							target,
							value: Expression::parse(lexer)?
						}.into()
					)
				}
			},
			_ => unreachable!("consume_one_of returned unexpected token"),
		};

		if lexer.peek().is_some() {
			lexer.consume(Token::Newline)?;
		}

		res
	}
}