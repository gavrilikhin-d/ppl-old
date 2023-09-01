extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Ranged, StartsHere, StringWithOffset};

/// AST for use statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Use {
    /// Offset of "use" keyword
    pub offset: usize,
    /// Path to introduce to current module
    pub path: Vec<StringWithOffset>,
}

impl Ranged for Use {
    fn start(&self) -> usize {
        self.offset
    }

    fn end(&self) -> usize {
        self.path.end()
    }
}

impl StartsHere for Use {
    /// Check that use statement starts at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::Use)
    }
}

impl Parse for Use {
    type Err = ParseError;

    /// Parse [`Use`] statement inside parsing context
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let offset = context.lexer.consume(Token::Use)?.start();

        let mut path = Vec::new();
        loop {
            path.push(context.consume_id()?);

            if context.lexer.consume(Token::Dot).is_err() {
                break;
            }
        }

        Ok(Use { offset, path })
    }
}
