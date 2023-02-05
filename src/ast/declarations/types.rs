extern crate ast_derive;
use ast_derive::AST;

use crate::{syntax::{error::ParseError, Lexer, Parse, StartsHere, StringWithOffset, Token, Context}, ast::TypeReference};

/// Member of type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Member {
	/// Name of member
	pub name: StringWithOffset,
	/// Type of member
	pub ty: TypeReference,
}

/// Parse single or multiple members, if they are separated by comma
pub fn parse_members(context: &mut Context<impl Lexer>)
	-> Result<Vec<Member>, ParseError> {
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

	Ok(names.into_iter().map(|name| Member { name, ty: ty.clone() }).collect())
}

/// Declaration of type
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct TypeDeclaration {
    /// Name of type
    pub name: StringWithOffset,
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

		let mut members = Vec::new();
		if context.lexer.consume(Token::Colon).is_ok() {
			members =
				context.parse_block(parse_members)?
					.into_iter()
					.flatten()
					.collect();
		} else {
			context.consume_eol()?;
		}

        Ok(TypeDeclaration { name, members })
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_type_without_body() {
		let type_decl =
			"type x"
				.parse::<TypeDeclaration>()
				.unwrap();
		assert_eq!(
			type_decl,
			TypeDeclaration {
				name: StringWithOffset::from("x").at(5),
				members: vec![],
			}
		);
	}

	#[test]
	fn test_type_with_body() {
		let type_decl =
			include_str!("../../../examples/point.ppl")
				.parse::<TypeDeclaration>()
				.unwrap();

		let ty = TypeReference {
			name: StringWithOffset::from("Integer").at(19),
		};
		assert_eq!(
			type_decl,
			TypeDeclaration {
				name: StringWithOffset::from("Point").at(5),
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