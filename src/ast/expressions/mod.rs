mod call;
pub use call::*;

mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod unary;
pub use unary::*;

mod tuple;
pub use tuple::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingExpression, ParseError},
    Lexer, Parse, Ranged, StartsHere, Token,
};

use derive_more::From;

use super::Declaration;

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    UnaryOperation(UnaryOperation),
    Call(Call),
	Tuple(Tuple),
}

impl StartsHere for Expression {
    /// Check that expression 100% starts at current lexer position
    fn starts_here(lexer: &mut impl Lexer) -> bool {
        !Declaration::starts_here(lexer) &&
		lexer.peek().map_or(
			false,
			|t| t != Token::Error && t != Token::Newline && t != Token::RParen
		)
    }
}

/// Parse atomic expression
pub(crate) fn parse_atomic_expression(lexer: &mut impl Lexer)
	-> Result<Expression, ParseError> {
	if Literal::starts_here(lexer) {
		return Ok(Literal::parse(lexer)?.into());
	}

	if Tuple::starts_here(lexer) {
		return Ok(Tuple::parse(lexer)?.into());
	}

	if VariableReference::starts_here(lexer) {
		return Ok(VariableReference::parse(lexer)?.into());
	}

	if UnaryOperation::starts_here(lexer) {
		return Ok(UnaryOperation::parse(lexer)?.into());
	}

	Err(MissingExpression {
		at: lexer.span().end.into(),
	}.into())
}

impl Parse for Expression {
    type Err = ParseError;

    /// Parse expression using lexer
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        if !Expression::starts_here(lexer) {
            return Err(MissingExpression {
                at: lexer.span().end.into(),
            }
            .into());
        }

		let call = Call::parse(lexer)?;
		if call.name_parts.len() > 1 {
			return Ok(call.into())
		}

		Ok(match call.name_parts.first().unwrap() {
			CallNamePart::Argument(arg) => arg.clone(),
			CallNamePart::Text(t) => VariableReference {
				name: t.clone(),
			}.into(),
		})
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
			Expression::Tuple(tuple) => tuple.range(),
        }
    }
}
