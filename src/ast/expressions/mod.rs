mod call;
pub use call::*;

mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod unary;
pub use unary::*;

mod binary;
pub use binary::*;

mod tuple;
pub use tuple::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingExpression, ParseError},
    Lexer, Parse, Ranged, StartsHere, Token, Context,
};

use derive_more::{From, TryInto};

use super::Declaration;

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, AST, Clone, From, TryInto)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    UnaryOperation(UnaryOperation),
	BinaryOperation(BinaryOperation),
    Call(Call),
	Tuple(Tuple),
}

impl StartsHere for Expression {
    /// Check that expression 100% starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        !Declaration::starts_here(context) &&
		context.lexer.peek().map_or(
			false,
			|t| t != Token::Error && t != Token::Newline && t != Token::RParen
		)
    }
}

/// Parse atomic expression
pub(crate) fn parse_atomic_expression(context: &mut Context<impl Lexer>)
	-> Result<Expression, ParseError> {
	if Literal::starts_here(context) {
		return Ok(Literal::parse(context)?.into());
	}

	if Tuple::starts_here(context) {
		return Ok(Tuple::parse(context)?.into());
	}

	if VariableReference::starts_here(context) {
		return Ok(VariableReference::parse(context)?.into());
	}

	if UnaryOperation::starts_here(context) {
		return Ok(UnaryOperation::parse(context)?.into());
	}

	Err(MissingExpression {
		at: context.lexer.span().end.into(),
	}.into())
}

/// Parse binary expression
pub(crate) fn parse_binary_expression(context: &mut Context<impl Lexer>)
	-> Result<Expression, ParseError> {
	let mut left = parse_atomic_expression(context)?;

	while context.lexer.peek().is_some_and(|t| t.is_operator()) {
		context.lexer.consume_one_of(
			&[Token::Operator, Token::Less, Token::Greater]
		)?;
		let operator = context.lexer.string_with_offset();

		let right = parse_atomic_expression(context)?;

		// TODO: handle precedence and associativity
		left = BinaryOperation {
			left: Box::new(left),
			operator,
			right: Box::new(right),
		}.into();
	}

	Ok(left)
}

impl Parse for Expression {
    type Err = ParseError;

    /// Parse expression using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if !Expression::starts_here(context) {
            return Err(MissingExpression {
                at: context.lexer.span().end.into(),
            }
            .into());
        }

		let call = Call::parse(context)?;
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
			Expression::BinaryOperation(op) => op.range(),
            Expression::Call(call) => call.range(),
			Expression::Tuple(tuple) => tuple.range(),
        }
    }
}
