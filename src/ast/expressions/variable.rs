extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Lexer, Parse, Ranged, StartsHere, StringWithOffset, Token,
};

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct VariableReference {
    /// Referenced variable name
    pub name: StringWithOffset,
}

impl StartsHere for VariableReference {
    /// Check that variable reference may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context
            .lexer
            .try_match(Token::Id)
            .is_ok_and(|t| t.as_str().chars().nth(0).is_some_and(|c| c.is_lowercase()))
    }
}

impl Parse for VariableReference {
    type Err = ParseError;

    /// Parse variable reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        Ok(VariableReference {
            name: context.lexer.consume(Token::Id)?,
        })
    }
}

impl Ranged for VariableReference {
    /// Get range of variable reference
    fn range(&self) -> std::ops::Range<usize> {
        self.name.range()
    }
}
