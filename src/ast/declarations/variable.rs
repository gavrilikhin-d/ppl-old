extern crate ast_derive;
use ast_derive::AST;

use crate::ast::{Expression, TypeReference};
use crate::mutability::{Mutability, Mutable};
use crate::syntax::error::{MissingVariableName, ParseError};
use crate::syntax::{Context, Identifier, Keyword, Lexer, Parse, Ranged, StartsHere, Token};

/// Declaration of the variable
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct VariableDeclaration {
    /// Keyword `let`
    pub keyword: Keyword<"let">,
    /// Name of variable
    pub name: Identifier,
    /// Type of variable
    pub ty: Option<TypeReference>,
    /// Initializer for variable
    pub initializer: Expression,

    /// Is this variable mutable
    pub mutability: Mutability,
}

impl Ranged for VariableDeclaration {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.initializer.end()
    }
}

impl Mutable for VariableDeclaration {
    fn is_mutable(&self) -> bool {
        self.mutability.is_mutable()
    }
}

impl StartsHere for VariableDeclaration {
    /// Check that variable declaration may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.try_match(Token::Let).is_ok()
    }
}

impl Parse for VariableDeclaration {
    type Err = ParseError;

    /// Parse variable declaration using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let keyword = context.consume_keyword::<"let">()?;

        let mutable = context.lexer.consume(Token::Mut).is_ok();

        let name = context.consume_id().or_else(|_| {
            Err(MissingVariableName {
                at: context.lexer.span().end.into(),
            })
        })?;

        let ty = if context.lexer.consume(Token::Colon).is_ok() {
            Some(TypeReference::parse(context)?)
        } else {
            None
        };

        context.lexer.consume(Token::Assign)?;

        let initializer = Expression::parse(context)?;

        context.consume_eol()?;

        Ok(VariableDeclaration {
            keyword,
            name,
            ty,
            initializer,
            mutability: match mutable {
                true => Mutability::Mutable,
                false => Mutability::Immutable,
            },
        })
    }
}

#[test]
fn test_variable_declaration() {
    let var = "let x = 1".parse::<VariableDeclaration>().unwrap();

    use crate::ast::Literal;
    assert_eq!(
        var,
        VariableDeclaration {
            keyword: Keyword::<"let">::at(0),
            name: Identifier::from("x").at(4),
            ty: None,
            initializer: Literal::Integer {
                offset: 8,
                value: "1".to_string()
            }
            .into(),
            mutability: Mutability::Immutable,
        }
    );

    let var = "let mut x = 1".parse::<VariableDeclaration>().unwrap();
    assert_eq!(
        var,
        VariableDeclaration {
            keyword: Keyword::<"let">::at(0),
            name: Identifier::from("x").at(8),
            ty: None,
            initializer: Literal::Integer {
                offset: 12,
                value: "1".to_string()
            }
            .into(),
            mutability: Mutability::Mutable,
        }
    );

    let var = "let x: Integer = 1".parse::<VariableDeclaration>().unwrap();
    assert_eq!(
        var,
        VariableDeclaration {
            keyword: Keyword::<"let">::at(0),
            name: Identifier::from("x").at(4).into(),
            ty: Some(TypeReference {
                name: Identifier::from("Integer").at(7).into(),
                generic_parameters: vec![]
            }),
            initializer: Literal::Integer {
                offset: 17,
                value: "1".to_string()
            }
            .into(),
            mutability: Mutability::Immutable,
        }
    );

    let var = "let mut x: Integer = 1"
        .parse::<VariableDeclaration>()
        .unwrap();
    assert_eq!(
        var,
        VariableDeclaration {
            keyword: Keyword::<"let">::at(0),
            name: Identifier::from("x").at(8),
            ty: Some(TypeReference {
                name: Identifier::from("Integer").at(11).into(),
                generic_parameters: vec![]
            }),
            initializer: Literal::Integer {
                offset: 21,
                value: "1".to_string()
            }
            .into(),
            mutability: Mutability::Mutable,
        }
    );
}
