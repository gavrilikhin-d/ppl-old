extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{StartsHere, Context};
use crate::syntax::{error::ParseError, Lexer, Parse, Token};

/// AST for assignment
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Assignment {
    /// Target to assign to
    pub target: Expression,
    /// Expression to assign
    pub value: Expression,
}

impl StartsHere for Assignment {
    /// Check that assignment may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        Expression::starts_here(context)
    }
}

impl Parse for Assignment {
    type Err = ParseError;

    /// Parse assignment using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let target = Expression::parse(context)?;

        context.lexer.consume(Token::Assign)?;

        let value = Expression::parse(context)?;

		context.consume_eol()?;

        Ok(Assignment { target, value })
    }
}
