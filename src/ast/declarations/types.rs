extern crate ast_derive;
use ast_derive::AST;

use crate::{
    ast::{Annotation, TypeReference},
    syntax::{
        error::ParseError, Context, Identifier, Lexer, Parse, StartsHere, Token,
    },
};

/// Member of type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member {
    /// Name of member
    pub name: Identifier,
    /// Type of member
    pub ty: TypeReference,
}

/// Parse single or multiple members, if they are separated by comma
pub fn parse_members(context: &mut Context<impl Lexer>) -> Result<Vec<Member>, ParseError> {
    let names = context.parse_comma_separated(|context| context.consume_id());

    context.lexer.consume(Token::Colon)?;

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

/// Declaration of a generic parameter
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct GenericParameter {
    /// Name of a generic parameter
    pub name: Identifier,
    /// Constraint for a generic parameter
    pub constraint: Option<TypeReference>,
}

impl Parse for GenericParameter {
    type Err = ParseError;

    /// Parse generic parameter using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let name = context.consume_id()?;

        let constraint = if context.lexer.consume(Token::Colon).is_ok() {
            Some(TypeReference::parse(context)?)
        } else {
            None
        };

        Ok(GenericParameter { name, constraint })
    }
}

/// Declaration of type
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeDeclaration {
    /// Annotations for type
    pub annotations: Vec<Annotation>,
    /// Name of type
    pub name: Identifier,
    /// Generic parameters of type
    pub generic_parameters: Vec<GenericParameter>,
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

        let name = context.consume_id()?;

        let mut generic_parameters = Vec::new();
        if context.lexer.consume(Token::Less).is_ok() {
            generic_parameters = context.parse_comma_separated(GenericParameter::parse);
            context.lexer.consume_greater()?;
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
                name: Identifier::from("x").at(5),
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
                name: Identifier::from("Point").at(5),
                generic_parameters: vec![GenericParameter {
                    name: Identifier::from("U").at(11),
                    constraint: None,
                }],
                members: vec![Member {
                    name: Identifier::from("x").at(16),
                    ty: TypeReference {
                        name: Identifier::from("U").at(18),
                        generic_parameters: Vec::new(),
                    },
                },],
            }
        );

        let type_decl = "type Point<U: A>:\n\tx:U"
            .parse::<TypeDeclaration>()
            .unwrap();
        assert_eq!(
            type_decl,
            TypeDeclaration {
                annotations: vec![],
                name: Identifier::from("Point").at(5),
                generic_parameters: vec![GenericParameter {
                    name: Identifier::from("U").at(11),
                    constraint: Some(TypeReference {
                        name: Identifier::from("A").at(14),
                        generic_parameters: Vec::new()
                    })
                }],
                members: vec![Member {
                    name: Identifier::from("x").at(19),
                    ty: TypeReference {
                        name: Identifier::from("U").at(21),
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
            name: Identifier::from("Integer").at(19),
            generic_parameters: Vec::new(),
        };
        assert_eq!(
            type_decl,
            TypeDeclaration {
                annotations: vec![],
                name: Identifier::from("Point").at(5),
                generic_parameters: vec![],
                members: vec![
                    Member {
                        name: Identifier::from("x").at(13),
                        ty: ty.clone(),
                    },
                    Member {
                        name: Identifier::from("y").at(16),
                        ty: ty.clone(),
                    },
                ],
            }
        );
    }
}
