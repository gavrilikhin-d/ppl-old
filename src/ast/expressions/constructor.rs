extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, Token, Context, StringWithOffset};

use super::{Expression, TypeReference};

/// Field initializer inside constructor
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Initializer {
	/// Name of member
	pub name: StringWithOffset,
	/// Value to initialize with
	pub value: Option<Expression>
}

impl Ranged for Initializer {
	fn start(&self) -> usize {
		self.name.start()
	}

	fn end(&self) -> usize {
		self.value.as_ref().map(|v| v.end()).unwrap_or(
			self.name.end()
		)
	}
}

impl Parse for Initializer {
	type Err = ParseError;

	fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
		let name = context.lexer.consume(Token::Id)?;

		let value = if context.lexer.consume(Token::Colon).is_ok() {
			Some(Expression::parse(context)?)
		} else {
			None
		};

		Ok(Initializer {
			name,
			value,
		})
	}
}

/// AST for object constructor
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Constructor {
	/// Type of constructed object
	pub ty: TypeReference,
	/// Offset of '{'
	pub lbrace: usize,
	/// Member initializers
	pub initializers: Vec<Initializer>,
	/// Offset of '}'
	pub rbrace: usize,
}

impl Ranged for Constructor {
	fn start(&self) -> usize {
		self.ty.start()
	}

	fn end(&self) -> usize {
		self.rbrace + 1
	}
}

impl Parse for Constructor {
	type Err = ParseError;

	fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
		let ty = TypeReference::parse(context)?;

		let lbrace = context.lexer.consume(Token::LBrace)?.start();
		let mut initializers = Vec::new();
		while context.lexer.peek() != Some(Token::RBrace) {
			initializers.push(Initializer::parse(context)?);

			if context.lexer.peek() == Some(Token::RBrace) {
				break;
			}

			context.lexer.consume(Token::Comma)?;
		}
		let rbrace = context.lexer.consume(Token::RBrace)?.start();

		Ok(Constructor {
			ty,
			lbrace,
			initializers,
			rbrace,
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::ast::Literal;

	use super::*;

	#[test]
	fn test_empty() {
		let res =
			"Empty {}"
				.parse::<Constructor>()
				.unwrap();
		assert_eq!(
			res,
			Constructor {
				ty: TypeReference {
					name: StringWithOffset::from("Empty"),
				},
				lbrace: 6,
				initializers: Vec::new(),
				rbrace: 7,
			}
		);
	}

	#[test]
	fn test_using_names() {
		let res =
			"Point {x, y}"
				.parse::<Constructor>()
				.unwrap();
		assert_eq!(
			res,
			Constructor {
				ty: TypeReference {
					name: StringWithOffset::from("Point"),
				},
				lbrace: 6,
				initializers: vec![
					Initializer {
						name: StringWithOffset::from("x").at(7),
						value: None,
					},
					Initializer {
						name: StringWithOffset::from("y").at(10),
						value: None,
					},
				],
				rbrace: 11,
			}
		);
	}

	#[test]
	fn test_using_values() {
		let res =
			"Point {x: 0, y: 0}"
				.parse::<Constructor>()
				.unwrap();
		assert_eq!(
			res,
			Constructor {
				ty: TypeReference {
					name: StringWithOffset::from("Point"),
				},
				lbrace: 6,
				initializers: vec![
					Initializer {
						name: StringWithOffset::from("x").at(7),
						value: Some(
							Literal::Integer {
								offset: 10,
								value: "0".to_string()
							}.into()
						),
					},
					Initializer {
						name: StringWithOffset::from("y").at(13),
						value: Some(
							Literal::Integer {
								offset: 16,
								value: "0".to_string()
							}.into()
						),
					},
				],
				rbrace: 17,
			}
		);
	}
}