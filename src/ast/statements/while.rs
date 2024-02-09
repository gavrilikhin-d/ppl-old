extern crate ast_derive;

use ast_derive::AST;

use crate::ast::{Expression, Statement};
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Keyword, Ranged, StartsHere};

/// AST for while loop
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct While {
    /// Keyword `while`
    pub keyword: Keyword<"while">,
    /// Condition of loop
    pub condition: Expression,
    /// Body of loop
    pub body: Vec<Statement>,
}

impl StartsHere for While {
    /// Check that loop starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::While)
    }
}

impl Parse for While {
    type Err = ParseError;

    /// Parse loop using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = Keyword::<"while">::at(context.lexer.consume(Token::While)?.start());

        let condition = Expression::parse(context)?;

        context.lexer.consume(Token::Colon)?;

        let body = context.parse_block(Statement::parse)?;

        Ok(While { keyword, condition, body })
    }
}
