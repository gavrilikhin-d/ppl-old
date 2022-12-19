use super::Statement;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{Lexer, Parse, error::ParseError};

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Module {
	/// Statements in module
	pub statements: Vec<Statement>,
}

impl Parse for Module {
	type Err = ParseError;

	/// Parse all statements in module
	fn parse(lexer: &mut Lexer) -> Result<Self, Self::Err> {
		let mut statements = Vec::new();

		while lexer.peek().is_some() {
			statements.push(Statement::parse(lexer)?);
		}

		Ok(Module { statements })
	}
}