mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod unary;
pub use unary::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, Ranged, error::{ParseError, MissingExpression}};

use derive_more::From;

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Expression {
	Literal(Literal),
	VariableReference(VariableReference),
	UnaryOperation(UnaryOperation)
}

impl Parse for Expression {
	type Err = ParseError;

	/// Parse expression using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(
			&[Token::None, Token::Integer, Token::Id, Token::Plus, Token::Minus]
		);
		if token.is_err() {
			return Err(
				MissingExpression {
					at: lexer.span().end.into()
				}.into()
			)
		}

		Ok(
			match token.unwrap() {
				Token::None | Token::Integer =>
					Expression::Literal(Literal::parse(lexer)?),
				Token::Id =>
					Expression::VariableReference(VariableReference::parse(lexer)?),
				Token::Plus | Token::Minus =>
					UnaryOperation::parse(lexer)?.into(),
				_ => unreachable!("consume_one_of returned unexpected token"),
			}
		)
	}
}

impl Ranged for Expression {
	/// Get range of expression
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Expression::Literal(l) => l.range(),
			Expression::VariableReference(var) => var.range(),
			Expression::UnaryOperation(op) => op.range(),
		}
	}
}

