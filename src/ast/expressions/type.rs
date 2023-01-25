extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, StringWithOffset, Token, Context};

/// AST for type reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeReference {
    /// Referenced type name
    pub name: StringWithOffset,
}

impl StartsHere for TypeReference {
    /// Check that type reference may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.try_match(Token::Id).is_ok()
    }
}

impl Parse for TypeReference {
    type Err = ParseError;

    /// Parse type reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        Ok(TypeReference {
            name: context.lexer.consume(Token::Id)?,
        })
    }
}

impl Ranged for TypeReference {
    /// Get range of type reference
    fn range(&self) -> std::ops::Range<usize> {
        self.name.range()
    }
}
