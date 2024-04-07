extern crate ast_derive;
use std::fmt::Display;

use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Identifier, Keyword, Lexer, Parse, Ranged, StartsHere, Token,
};

use derive_more::From;

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Typename {
    Identifier(Identifier),
    Reference {
        ampersand: Keyword<"&">,
        mutable: Option<Keyword<"mut">>,
    },
}

impl Display for Typename {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typename::Identifier(name) => write!(f, "{name}"),
            Typename::Reference { mutable, .. } => {
                write!(f, "&{}", mutable.map_or("", |_| "mut"))
            }
        }
    }
}

impl Ranged for Typename {
    fn start(&self) -> usize {
        match self {
            Typename::Identifier(name) => name.start(),
            Typename::Reference { ampersand, .. } => ampersand.start(),
        }
    }

    fn end(&self) -> usize {
        match self {
            Typename::Identifier(name) => name.end(),
            Typename::Reference { mutable, ampersand } => mutable
                .as_ref()
                .map_or_else(|| ampersand.end(), |m| m.end()),
        }
    }
}

/// AST for type reference
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeReference {
    /// Referenced type name
    pub name: Typename,
    /// Generic parameters
    pub generic_parameters: Vec<TypeReference>,
}

impl StartsHere for TypeReference {
    /// Check that type reference may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context
            .lexer
            .try_match_one_of(&[Token::Id, Token::EscapedId])
            .is_ok_and(|_| {
                Identifier::from(context.lexer.peek_string_with_offset())
                    .as_str()
                    .chars()
                    .nth(0)
                    .is_some_and(|c| c.is_uppercase())
            })
    }
}

impl Parse for TypeReference {
    type Err = ParseError;

    /// Parse type reference using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if let Ok(ampersand) = context.consume_keyword::<"&">() {
            let mutable = context.consume_keyword::<"mut">().ok();
            return Ok(TypeReference {
                name: Typename::Reference { ampersand, mutable },
                generic_parameters: vec![TypeReference::parse(context)?],
            });
        }

        let name = context.consume_id()?;
        let mut generic_parameters = Vec::new();
        if context.lexer.consume(Token::Less).is_ok() {
            loop {
                generic_parameters.push(TypeReference::parse(context)?);
                if context.lexer.consume(Token::Comma).is_err() {
                    break;
                }
            }
            context.lexer.consume_greater()?;
        }

        Ok(TypeReference {
            name: name.into(),
            generic_parameters,
        })
    }
}

impl Ranged for TypeReference {
    /// Get range of type reference
    fn range(&self) -> std::ops::Range<usize> {
        self.name.start()
            ..self
                .generic_parameters
                .last()
                .map_or(self.name.end(), |p| p.range().end)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    #[test]
    fn without_generics() {
        use super::*;

        let res = "Foo".parse::<TypeReference>();
        assert_eq!(
            res,
            Ok(TypeReference {
                name: Identifier::from("Foo").into(),
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
                name: Identifier::from("Foo").at(0).into(),
                generic_parameters: vec![
                    TypeReference {
                        name: Identifier::from("Bar").at(4).into(),
                        generic_parameters: Vec::new(),
                    },
                    TypeReference {
                        name: Identifier::from("Baz").at(9).into(),
                        generic_parameters: Vec::new(),
                    },
                ],
            })
        );
    }

    #[test]
    fn reference_generic_with_generic() {
        use super::*;

        let res = "Foo<Bar<Baz>>".parse::<TypeReference>();
        assert_eq!(
            res,
            Ok(TypeReference {
                name: Identifier::from("Foo").at(0).into(),
                generic_parameters: vec![TypeReference {
                    name: Identifier::from("Bar").at(4).into(),
                    generic_parameters: [TypeReference {
                        name: Identifier::from("Baz").at(8).into(),
                        generic_parameters: Vec::new(),
                    }]
                    .into(),
                },],
            })
        );
    }
}
