extern crate ast_derive;
use ast_derive::AST;

use super::{Expression, parse_binary_expression};
use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StringWithOffset, Context};


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

    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let expr = parse_binary_expression(context)?;

		if !matches!(expr, Expression::BinaryOperation(_)) {
			todo!("expected binary expression error")
		}

		Ok(expr.try_into().unwrap())
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
