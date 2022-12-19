mod function;
pub use function::*;

mod types;
pub use types::*;

mod variable;
pub use variable::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, error::ParseError};

use derive_more::From;


/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Declaration {
	Variable(VariableDeclaration),
	Type(TypeDeclaration),
	Function(FunctionDeclaration),
}

impl Parse for Declaration {
	type Err = ParseError;

	/// Parse declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.try_match_one_of(
			&[Token::Type, Token::Let, Token::Fn]
		)?;
		match token {
			Token::Type =>
				TypeDeclaration::parse(lexer).map(Declaration::Type),
			Token::Let =>
				VariableDeclaration::parse(lexer).map(Declaration::Variable),
			Token::Fn =>
				FunctionDeclaration::parse(lexer).map(Declaration::Function),
			_ => unreachable!("try_ match_one_of returned unexpected token"),
		}
	}
}