mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{
    error::{MissingDeclaration, ParseError},
    Lexer, Parse, StartsHere, Token,
};

use derive_more::From;

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Declaration {
    Variable(VariableDeclaration),
    Type(TypeDeclaration),
    Function(FunctionDeclaration),
}

impl StartsHere for Declaration {
    /// Check literal may start at current lexer position
    fn starts_here(lexer: &mut impl Lexer) -> bool {
        VariableDeclaration::starts_here(lexer)
            || TypeDeclaration::starts_here(lexer)
            || FunctionDeclaration::starts_here(lexer)
    }
}

impl Parse for Declaration {
    type Err = ParseError;

    /// Parse declaration using lexer
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        if !Declaration::starts_here(lexer) {
            return Err(MissingDeclaration {
                at: lexer.span().end.into(),
            }
            .into());
        }

        match lexer.peek().unwrap() {
            Token::Type => TypeDeclaration::parse(lexer).map(Declaration::Type),
            Token::Let => VariableDeclaration::parse(lexer).map(Declaration::Variable),
            Token::Fn => FunctionDeclaration::parse(lexer).map(Declaration::Function),
            _ => unreachable!("unexpected token in start of declaration"),
        }
    }
}
