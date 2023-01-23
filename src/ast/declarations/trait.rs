extern crate ast_derive;
use ast_derive::AST;

use crate::{syntax::{error::ParseError, Lexer, Parse, StartsHere, StringWithOffset, Token, Context}, ast::Statement};

use super::{FunctionDeclaration, Declaration};

/// Declaration of trait
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TraitDeclaration {
    /// Name of trait
    pub name: StringWithOffset,
	/// Associated functions
	pub functions: Vec<FunctionDeclaration>
}

impl StartsHere for TraitDeclaration {
    /// Check that type declaration may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::Trait)
    }
}

impl Parse for TraitDeclaration {
    type Err = ParseError;

    /// Parse trait declaration
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        context.lexer.consume(Token::Trait)?;

        let name = context.lexer.consume(Token::Id)?;

		context.lexer.consume(Token::Colon)?;

		let functions = context.parse_block()?.into_iter().map(|decl| {
			match decl {
				Statement::Declaration(Declaration::Function(func))
					=> Ok::<FunctionDeclaration, ParseError>(func),
				_ => todo!("error on wrong declaration in trait body")
			}
		}).try_collect::<Vec<_>>()?;

        Ok(TraitDeclaration { name, functions })
    }
}
