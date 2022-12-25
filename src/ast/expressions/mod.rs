mod call;
pub use call::*;

mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod unary;
pub use unary::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, Ranged, error::{ParseError, MissingExpression}, StartsHere};

use derive_more::From;

use super::Declaration;

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Expression {
	Literal(Literal),
	VariableReference(VariableReference),
	UnaryOperation(UnaryOperation),
	Call(Call),
}

impl StartsHere for Expression {
	/// Check that expression 100% starts at current lexer position
	fn starts_here(lexer: &mut Lexer) -> bool {
		!Declaration::starts_here(lexer) &&
		lexer.peek().map_or(false, |t| t != Token::Error)
	}
}

impl Parse for Expression {
	type Err = ParseError;

	/// Parse expression using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		if !Expression::starts_here(lexer) {
			return Err(
				MissingExpression {
					at: lexer.span().end.into()
				}.into()
			)
		}

		if Literal::starts_here(lexer) {
			return Ok(Literal::parse(lexer)?.into());
		}

		Ok(
			match lexer.peek().unwrap() {
				Token::Id => {
					let call = Call::parse(lexer)?;
					if call.name_parts.len() > 1 {
						call.into()
					}
					else
					{
						VariableReference {
							name: call.name_parts.first().unwrap().clone().try_into().unwrap()
						}.into()
					}

				}
				Token::Plus | Token::Minus =>
					UnaryOperation::parse(lexer)?.into(),
				_ => unreachable!("unexpected token at start of expression"),
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
			Expression::Call(call) => call.range(),
		}
	}
}

