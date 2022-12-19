extern crate ast_derive;
use ast_derive::AST;

use crate::mutability::{Mutability, Mutable};
use crate::ast::Expression;
use crate::syntax::error::{ParseError, MissingVariableName};
use crate::syntax::{Token, Lexer, Parse, StringWithOffset};

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

impl Parse for VariableDeclaration {
	type Err = ParseError;

	/// Parse variable declaration using lexer
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		lexer.consume(Token::Let)?;

		let mutable = lexer.consume(Token::Mut).is_ok();

		lexer.consume(Token::Id).or_else(
			|_| Err(MissingVariableName {
				at: lexer.span().end.into()
			})
		)?;

		let name = StringWithOffset::from(lexer.slice()).at(lexer.span().start);

		lexer.consume(Token::Assign)?;

		Ok(
			VariableDeclaration {
				name,
				initializer: Expression::parse(lexer)?,
				mutability: match mutable {
					true => Mutability::Mutable,
					false => Mutability::Immutable
				}
			}
		)
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
			initializer: Literal::Integer { offset: 8, value: "1".to_string() }.into(),
			mutability: Mutability::Immutable,
		}
	);

	let var = "let mut x = 1".parse::<VariableDeclaration>().unwrap();
	assert_eq!(
		var,
		VariableDeclaration {
			name: StringWithOffset::from("x").at(8),
			initializer: Literal::Integer { offset: 12, value: "1".to_string() }.into(),
			mutability: Mutability::Mutable,
		}
	)
}