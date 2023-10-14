extern crate ast_derive;
use ast_derive::AST;

use crate::{
    ast::{Annotation, TypeReference},
    syntax::{error::ParseError, Context, Lexer, Parse, StartsHere, StringWithOffset, Token},
};

/// Member of type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member {
    /// Name of member
    pub name: StringWithOffset,
    /// Type of member
    pub ty: TypeReference,
}

/// Parse single or multiple members, if they are separated by comma
pub fn parse_members(context: &mut Context<impl Lexer>) -> Result<Vec<Member>, ParseError> {
    let mut names = Vec::new();
    loop {
        names.push(context.lexer.consume(Token::Id)?);

        if context.lexer.consume(Token::Colon).is_ok() {
            break;
        }

        context.lexer.consume(Token::Comma)?;
    }

    let ty = TypeReference::parse(context)?;

    context.consume_eol()?;

    Ok(names
        .into_iter()
        .map(|name| Member {
            name,
            ty: ty.clone(),
        })
        .collect())
}

/// Declaration of type
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeDeclaration {
    /// Annotations for type
    pub annotations: Vec<Annotation>,
    /// Name of type
    pub name: StringWithOffset,
    /// Generic parameters of type
    pub generic_parameters: Vec<StringWithOffset>,
    /// Members of type
    pub members: Vec<Member>,
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

        let mut generic_parameters = Vec::new();
        if context.lexer.consume(Token::Less).is_ok() {
            loop {
                generic_parameters.push(context.lexer.consume(Token::Id)?);

                if context.lexer.consume_greater().is_ok() {
                    break;
                }

                context.lexer.consume(Token::Comma)?;
            }
        }

        let mut members = Vec::new();
        if context.lexer.consume(Token::Colon).is_ok() {
            members = context
                .parse_block(parse_members)?
                .into_iter()
                .flatten()
                .collect();
        } else {
            context.consume_eol()?;
        }

        Ok(TypeDeclaration {
            annotations: vec![],
            name,
            generic_parameters,
            members,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_type_without_body() {
        let type_decl = "type x".parse::<TypeDeclaration>().unwrap();
        assert_eq!(
            type_decl,
            TypeDeclaration {
                annotations: vec![],
                name: StringWithOffset::from("x").at(5),
                generic_parameters: vec![],
                members: vec![],
            }
        );
    }

    #[test]
    fn type_with_generic_parameters() {
        let type_decl = "type Point<U>:\n\tx:U".parse::<TypeDeclaration>().unwrap();
        assert_eq!(
            type_decl,
            TypeDeclaration {
                annotations: vec![],
                name: StringWithOffset::from("Point").at(5),
                generic_parameters: vec![StringWithOffset::from("U").at(11),],
                members: vec![Member {
                    name: StringWithOffset::from("x").at(16),
                    ty: TypeReference {
                        name: StringWithOffset::from("U").at(18),
                        generic_parameters: Vec::new(),
                    },
                },],
            }
        )
    }

    #[test]
    fn test_type_with_body() {
        let type_decl = include_str!("../../../examples/point.ppl")
            .parse::<TypeDeclaration>()
            .unwrap();

        let ty = TypeReference {
            name: StringWithOffset::from("Integer").at(19),
            generic_parameters: Vec::new(),
        };
        assert_eq!(
            type_decl,
            TypeDeclaration {
                annotations: vec![],
                name: StringWithOffset::from("Point").at(5),
                generic_parameters: vec![],
                members: vec![
                    Member {
                        name: StringWithOffset::from("x").at(13),
                        ty: ty.clone(),
                    },
                    Member {
                        name: StringWithOffset::from("y").at(16),
                        ty: ty.clone(),
                    },
                ],
            }
        );
    }
}
