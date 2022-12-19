extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Token, Lexer, Parse, StringWithOffset, error::ParseError};

/// Declaration of type
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeDeclaration {
	/// Name of type
	pub name: StringWithOffset,
}

impl Parse for TypeDeclaration {
	type Err = ParseError;

	/// Parse type declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Type)?;

		lexer.consume(Token::Id)?;

		let name = lexer.string_with_offset();

		Ok(TypeDeclaration {name})
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