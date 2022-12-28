extern crate ast_derive;
use ast_derive::AST;

use derive_more::From;

use crate::{syntax::{Token, Lexer, Parse, StringWithOffset, error::ParseError, StartsHere}, ast::{Annotation, Statement}};

/// Parameter of function
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Parameter {
	/// Parameter's name
	pub name: StringWithOffset,
	/// Parameter's type
	pub ty: StringWithOffset,
}

impl Parse for Parameter {
	type Err = ParseError;

	/// Parse parameter using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let name = lexer.consume(Token::Id)?;

		lexer.consume(Token::Colon)?;

		let ty = lexer.consume(Token::Id)?;

		Ok(
			Parameter {
				name,
				ty,
			}
		)
	}
}

/// Cell of function
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum FunctionNamePart {
	Text(StringWithOffset),
	Parameter(Parameter),
}

impl Parse for FunctionNamePart {
	type Err = ParseError;

	/// Parse function name part using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let token = lexer.consume_one_of(&[Token::Id, Token::Less])?;
		match token {
			Token::Id =>
				Ok(lexer.string_with_offset().into()),
			Token::Less => {
				let p = Parameter::parse(lexer)?;

				lexer.consume(Token::Greater)?;

				Ok(p.into())
			}
			_ => unreachable!("consume_one_of returned unexpected token"),
		}
	}
}

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct FunctionDeclaration {
	/// Name parts of function
	pub name_parts: Vec<FunctionNamePart>,
	/// Return type of function
	pub return_type: Option<StringWithOffset>,
	/// Body of function
	pub body: Option<Vec<Statement>>,

	/// Annotations for function
	pub annotations: Vec<Annotation>
}

impl StartsHere for FunctionDeclaration {
	/// Check that function declaration may start at current lexer position
	fn starts_here(lexer: &mut Lexer) -> bool {
		lexer.try_match(Token::Fn).is_ok()
	}
}

impl Parse for FunctionDeclaration {
	type Err = ParseError;

	/// Parse function declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Fn)?;

		let mut name_parts = Vec::new();

		loop {
			let part = FunctionNamePart::parse(lexer)?;
			name_parts.push(part);

			match lexer.peek() {
				None | Some(Token::Arrow) | Some(Token::Newline) => break,
				_ => continue,
			}
		}

		let return_type = if lexer.consume(Token::Arrow).is_ok() {
			Some(lexer.consume(Token::Id)?)
		} else {
			None
		};

		let mut body = None;

		if lexer.consume(Token::Colon).is_ok() {
			lexer.consume(Token::Newline)?;

			let mut stmts = Vec::new();

			let indentation = lexer.indentation() + 1;
			loop {
				lexer.skip_indentation();
				stmts.push(Statement::parse(lexer)?);
				if lexer.indentation() < indentation {
					break;
				}
			}

			body = Some(stmts);
		}

		Ok(FunctionDeclaration {
			name_parts,
			return_type,
			body,
			annotations: vec![]
		})
	}

}

#[test]
fn test_function_declaration() {
	let func =
		"fn distance from <a: Point> to <b: Point> -> Distance"
			.parse::<FunctionDeclaration>()
			.unwrap();
	assert_eq!(
		func,
		FunctionDeclaration {
			name_parts: vec![
				StringWithOffset::from("distance").at(3).into(),
				StringWithOffset::from("from").at(12).into(),
				Parameter {
					name: StringWithOffset::from("a").at(18).into(),
					ty: StringWithOffset::from("Point").at(21).into(),
				}.into(),
				StringWithOffset::from("to").at(28).into(),
				Parameter {
					name: StringWithOffset::from("b").at(32).into(),
					ty: StringWithOffset::from("Point").at(35).into(),
				}.into(),
			],
			return_type: Some(
				StringWithOffset::from("Distance").at(45).into()
			)
		}
	);
}