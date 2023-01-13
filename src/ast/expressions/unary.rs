extern crate ast_derive;

use ast_derive::AST;

use super::{Expression, parse_atomic_expression};
use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, Token, WithOffset, StringWithOffset, Context};

/// Kind of unary operator
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperatorKind {
    Prefix,
    Postfix,
}

/// AST for unary expression
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct UnaryOperation {
    /// Operator of unary expression
    pub operator: StringWithOffset,
    /// Operand of unary expression
    pub operand: Box<Expression>,

    /// Kind of unary operator
    pub kind: UnaryOperatorKind,
}

impl UnaryOperation {
    /// Get name format of function associated with unary operator
    pub fn name_format(&self) -> String {
        match self.kind {
            UnaryOperatorKind::Prefix => format!("{} <>", self.operator),
            UnaryOperatorKind::Postfix => format!("<> {}", self.operator),
        }
    }
}

impl StartsHere for UnaryOperation {
    /// Check that unary operation may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek().is_some_and(|t| t.is_operator())
    }
}

impl Parse for UnaryOperation {
    type Err = ParseError;

    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
		// TODO: postfix expressions
        let operator = context.lexer.consume(Token::Operator)?;

        let operand = parse_atomic_expression(context)?;

        Ok(UnaryOperation {
            operand: Box::new(operand),
            operator,
            kind: UnaryOperatorKind::Prefix,
        })
    }
}

impl Ranged for UnaryOperation {
    fn start(&self) -> usize {
        use UnaryOperatorKind::*;
        match self.kind {
            Prefix => self.operator.start(),
            Postfix => self.operand.start(),
        }
    }

    fn end(&self) -> usize {
        use UnaryOperatorKind::*;
        match self.kind {
            Prefix => self.operand.end(),
            Postfix => self.operator.end(),
        }
    }
}
