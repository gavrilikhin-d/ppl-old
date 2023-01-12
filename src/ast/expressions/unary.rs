extern crate ast_derive;
use std::fmt::Display;

use ast_derive::AST;

use super::{Expression, parse_atomic_expression};
use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, Token, WithOffset};

/// Unary operators
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    /// '+'
    Plus,
    /// '-'
    Minus,
}

impl UnaryOperator {
    /// Get length of operator
    pub fn len(&self) -> usize {
        1
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Plus => write!(f, "+"),
            UnaryOperator::Minus => write!(f, "-"),
        }
    }
}

impl Ranged for WithOffset<UnaryOperator> {
    fn range(&self) -> std::ops::Range<usize> {
        self.offset..(self.offset + self.value.len())
    }
}

impl TryFrom<Token> for UnaryOperator {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        Ok(match value {
            Token::Plus => UnaryOperator::Plus,
            Token::Minus => UnaryOperator::Minus,
            _ => return Err(()),
        })
    }
}

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
    pub operator: WithOffset<UnaryOperator>,
    /// Operand of unary expression
    pub operand: Box<Expression>,

    /// Kind of unary operator
    pub kind: UnaryOperatorKind,
}

impl UnaryOperation {
    /// Get name format of function associated with unary operator
    pub fn name_format(&self) -> String {
        match self.kind {
            UnaryOperatorKind::Prefix => format!("{} <>", self.operator.value),
            UnaryOperatorKind::Postfix => format!("<> {}", self.operator.value),
        }
    }
}

impl StartsHere for UnaryOperation {
    /// Check that unary operation may start at current lexer position
    fn starts_here(lexer: &mut impl Lexer) -> bool {
        lexer.try_match_one_of(&[Token::Plus, Token::Minus]).is_ok()
    }
}

impl Parse for UnaryOperation {
    type Err = ParseError;

    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        let prefix = lexer.consume_one_of(&[Token::Plus, Token::Minus])?;

        let offset = lexer.span().start;

        let operand = parse_atomic_expression(lexer)?;

        Ok(UnaryOperation {
            operand: Box::new(operand),
            operator: WithOffset {
                offset,
                value: prefix.try_into().unwrap(),
            },
            kind: UnaryOperatorKind::Prefix,
        })
    }
}

impl Ranged for UnaryOperation {
    fn start(&self) -> usize {
        use UnaryOperatorKind::*;
        match self.kind {
            Prefix => self.operator.offset,
            Postfix => self.operand.start(),
        }
    }

    fn end(&self) -> usize {
        use UnaryOperatorKind::*;
        match self.kind {
            Prefix => self.operand.end(),
            Postfix => self.operator.offset,
        }
    }
}
