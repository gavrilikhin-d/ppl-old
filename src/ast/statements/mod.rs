mod assignment;
pub use assignment::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::ast::{Declaration, Expression};
use crate::syntax::StartsHere;
use crate::syntax::error::MissingStatement;
use crate::syntax::{Token, Lexer, Parse, error::ParseError};

use derive_more::From;

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Statement {
	Declaration(Declaration),
	Expression(Expression),
	Assignment(Assignment),
}

impl StartsHere for Statement {
	/// Check that statement may start at current lexer position
	fn starts_here(lexer: &mut Lexer) -> bool {
		Declaration::starts_here(lexer) ||
		Expression::starts_here(lexer) ||
		Assignment::starts_here(lexer)
	}
}

impl Parse for Statement {
	type Err = ParseError;

	/// Parse statement using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.skip_spaces();

		if !Statement::starts_here(lexer) {
			return Err(
				MissingStatement {
					at: lexer.span().end.into()
				}.into()
			);
		}

		let res = match lexer.peek().unwrap() {
			Token::Let | Token::Type | Token::Fn =>
				Declaration::parse(lexer)
					.map(|decl| Statement::Declaration(decl)),
			Token::None | Token::Integer | Token::String |
			Token::Id | Token::Plus | Token::Minus => {
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