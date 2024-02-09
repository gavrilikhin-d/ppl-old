extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Keyword, Ranged, StartsHere};

/// AST for return statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Return {
    /// Keyword `return`
    pub keyword: Keyword<"return">,
    /// Returned value
    pub value: Option<Expression>,
}

impl Ranged for Return {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.value.as_ref().map_or(self.keyword.end(), |v| v.end())
    }
}

impl StartsHere for Return {
    /// Check that return may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::Return)
    }
}

impl Parse for Return {
    type Err = ParseError;

    /// Parse return using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"return">()?;

        let value = if Expression::starts_here(context) {
            Some(Expression::parse(context)?)
        } else {
            None
        };

        Ok(Return { keyword, value })
    }
}
