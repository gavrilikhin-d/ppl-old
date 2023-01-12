extern crate ast_derive;
use std::fmt::Display;

use ast_derive::AST;

use super::{Expression, parse_atomic_expression};
use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, Token, WithOffset, StringWithOffset};


/// AST for unary expression
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct BinaryOperation {
	/// Left operand
	pub left: Box<Expression>,
    /// Operator of unary expression
    pub operator: StringWithOffset,
	/// Right operand
	pub right: Box<Expression>,
}

impl BinaryOperation {
    /// Get name format of function associated with unary operator
    pub fn name_format(&self) -> String {
        format!("<> {} <>", self.operator)
    }
}

impl Parse for BinaryOperation {
    type Err = ParseError;

    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        let mut left = parse_atomic_expression(lexer)?;

		loop {
			lexer.consume_one_of(
				&[Token::Operator, Token::Less, Token::Greater]
			)?;
			let operator = lexer.string_with_offset();

			let right = parse_atomic_expression(lexer)?;

			// TODO: handle precedence and associativity
			left = BinaryOperation {
				left: Box::new(left),
				operator,
				right: Box::new(right),
			}.into();

			if !lexer.peek().is_some_and(|t| t.is_operator()) {
				break;
			}
		}

		Ok(left.try_into().unwrap())
    }
}

impl Ranged for BinaryOperation {
    fn start(&self) -> usize {
        self.left.start()
    }

    fn end(&self) -> usize {
        self.right.end()
    }
}
