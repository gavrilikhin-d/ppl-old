extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Identifier, Ranged, StartsHere};

/// AST for use statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Use {
    /// Offset of "use" keyword
    pub offset: usize,
    /// Path to introduce to current module
    pub path: Vec<Identifier>,
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

        let path = context.parse_separated(|context| context.consume_id(), Token::Dot);

        Ok(Use { offset, path })
    }
}
