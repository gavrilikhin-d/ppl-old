extern crate ast_derive;

use ast_derive::AST;

use crate::ast::Statement;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Keyword, StartsHere};

/// AST for infinite loop
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Loop {
    pub keyword: Keyword<"loop">,
    /// Body of loop
    pub body: Vec<Statement>,
}

impl StartsHere for Loop {
    /// Check that loop starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::Loop)
    }
}

impl Parse for Loop {
    type Err = ParseError;

    /// Parse loop using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"loop">()?;

        context.lexer.consume(Token::Colon)?;

        let body = context.parse_block(Statement::parse)?;

        Ok(Loop { keyword, body })
    }
}
