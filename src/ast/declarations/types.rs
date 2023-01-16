extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, StartsHere, StringWithOffset, Token, Context};

/// Declaration of type
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeDeclaration {
    /// Name of type
    pub name: StringWithOffset,
}

impl StartsHere for TypeDeclaration {
    /// Check that type declaration may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.try_match(Token::Type).is_ok()
    }
}

impl Parse for TypeDeclaration {
    type Err = ParseError;

    /// Parse type declaration using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        context.lexer.consume(Token::Type)?;

        let name = context.lexer.consume(Token::Id)?;

		context.consume_eol()?;

        Ok(TypeDeclaration { name })
    }
}

#[test]
fn test_type() {
    let type_decl = "type x".parse::<TypeDeclaration>().unwrap();
    assert_eq!(
        type_decl,
        TypeDeclaration {
            name: StringWithOffset::from("x").at(5)
        }
    );
}
