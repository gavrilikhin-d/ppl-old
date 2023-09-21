extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::mutability::{Mutability, Mutable};
use crate::syntax::error::{MissingVariableName, ParseError};
use crate::syntax::{Context, Lexer, Parse, StartsHere, StringWithOffset, Token};

/// Declaration of the variable
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct VariableDeclaration {
    /// Name of variable
    pub name: StringWithOffset,
    /// Initializer for variable
    pub initializer: Expression,

    /// Is this variable mutable
    pub mutability: Mutability,
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
        context.lexer.consume(Token::Let)?;

        let mutable = context.lexer.consume(Token::Mut).is_ok();

        let name = context.lexer.consume(Token::Id).or_else(|_| {
            Err(MissingVariableName {
                at: context.lexer.span().end.into(),
            })
        })?;

        context.lexer.consume(Token::Assign)?;

        let initializer = Expression::parse(context)?;

        context.consume_eol()?;

        Ok(VariableDeclaration {
            name,
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
            name: StringWithOffset::from("x").at(4),
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
            name: StringWithOffset::from("x").at(8),
            initializer: Literal::Integer {
                offset: 12,
                value: "1".to_string()
            }
            .into(),
            mutability: Mutability::Mutable,
        }
    )
}
