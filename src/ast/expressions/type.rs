extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Lexer, Parse, Ranged, StartsHere, StringWithOffset, Token,
};

/// AST for type reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeReference {
    /// Referenced type name
    pub name: StringWithOffset,
    /// Generic parameters
    pub generic_parameters: Vec<TypeReference>,
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
        let name = context.lexer.consume(Token::Id)?;

        let mut generic_parameters = Vec::new();
        if context.lexer.consume(Token::Less).is_ok() {
            loop {
                generic_parameters.push(TypeReference::parse(context)?);
                if context.lexer.consume(Token::Comma).is_err() {
                    break;
                }
            }
            context.lexer.consume(Token::Greater)?;
        }

        Ok(TypeReference {
            name,
            generic_parameters,
        })
    }
}

impl Ranged for TypeReference {
    /// Get range of type reference
    fn range(&self) -> std::ops::Range<usize> {
        self.name.range()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn without_generics() {
        use super::*;

        let res = "Foo".parse::<TypeReference>();
        assert_eq!(
            res,
            Ok(TypeReference {
                name: StringWithOffset::from("Foo"),
                generic_parameters: Vec::new(),
            })
        );
    }

    #[test]
    fn with_generics() {
        use super::*;

        let res = "Foo<Bar, Baz>".parse::<TypeReference>();
        assert_eq!(
            res,
            Ok(TypeReference {
                name: StringWithOffset::from("Foo").at(0),
                generic_parameters: vec![
                    TypeReference {
                        name: StringWithOffset::from("Bar").at(4),
                        generic_parameters: Vec::new(),
                    },
                    TypeReference {
                        name: StringWithOffset::from("Baz").at(9),
                        generic_parameters: Vec::new(),
                    },
                ],
            })
        );
    }
}
