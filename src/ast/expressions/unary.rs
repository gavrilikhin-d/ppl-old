extern crate ast_derive;
use ast_derive::AST;

use super::Expression;
use crate::syntax::{Token, Lexer, Parse, Ranged, WithOffset, error::ParseError};

/// Unary operators
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
	/// '+'
	Plus,
	/// '-'
	Minus
}

impl TryFrom<Token> for UnaryOperator {
	type Error = ();

	fn try_from(value: Token) -> Result<Self, Self::Error> {
		Ok(
			match value {
				Token::Plus => UnaryOperator::Plus,
				Token::Minus => UnaryOperator::Minus,
				_ => return Err(())
			}
		)
	}
}

/// Kind of unary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperatorKind
{
	Prefix,
	Postfix
}

/// AST for unary expression
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct UnaryOperation {
	/// Operator of unary expression
	pub operator: WithOffset<UnaryOperator>,
	/// Operand of unary expression
	pub operand: Box<Expression>,

	/// Kind of unary operator
	pub kind: UnaryOperatorKind
}

impl Parse for UnaryOperation {
	type Err = ParseError;

	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let prefix = lexer.consume_one_of(&[Token::Plus, Token::Minus])?;

		let offset = lexer.span().start;

		let operand = Expression::parse(lexer)?;

		Ok(UnaryOperation {
			operand: Box::new(operand),
			operator: WithOffset {
				offset,
				value: prefix.try_into().unwrap()
			},
			kind: UnaryOperatorKind::Prefix
		})
	}
}

impl Ranged for UnaryOperation {
	fn start(&self) -> usize {
		use UnaryOperatorKind::*;
		match self.kind {
			Prefix => self.operator.offset,
			Postfix => self.operand.start()
		}
	}

	fn end(&self) -> usize {
		use UnaryOperatorKind::*;
		match self.kind {
			Prefix => self.operand.end(),
			Postfix => self.operator.offset
		}
	}
}