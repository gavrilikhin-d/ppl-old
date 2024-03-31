extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Token};
use crate::syntax::{Context, Identifier, Keyword, Ranged, StartsHere};

/// AST for use statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Use {
    /// Keyword `use`
    pub keyword: Keyword<"use">,
    /// Path to introduce to current module
    pub path: Vec<Identifier>,
}

impl Use {
    /// Generate use statement for builtin module
    pub fn builtin_module() -> Self {
        Self {
            keyword: Keyword::<"use">::at(0),
            path: vec!["ppl".into(), "*".into()],
        }
    }
}

impl Ranged for Use {
    fn start(&self) -> usize {
        self.keyword.start()
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
        let keyword = context.consume_keyword::<"use">()?;

        let path = context.parse_separated(
            |context| {
                context
                    .consume_id()
                    .or_else(|_| context.lexer.consume(Token::Star).map(Into::into))
            },
            Token::Dot,
        );

        Ok(Use { keyword, path })
    }
}
