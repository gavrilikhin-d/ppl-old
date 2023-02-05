extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{StartsHere, Context};
use crate::syntax::{error::ParseError, Lexer, Parse, Token};

use super::Statement;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElseIf {
	/// Condition of else-if statement
	pub condition: Expression,
	/// Body of else-if statement
	pub body: Vec<Statement>,
}

/// AST for if-statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct If {
	/// Condition of if-statement
    pub condition: Expression,
	/// Body of if-statement
	pub body: Vec<Statement>,
	/// Else-if statements
	pub else_ifs: Vec<ElseIf>,
	/// Else block
	pub else_block: Vec<Statement>,
}

impl StartsHere for If {
    /// Check that assignment may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::If)
    }
}

impl Parse for If {
    type Err = ParseError;

    /// Parse assignment using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        context.lexer.consume(Token::If)?;

		let condition = Expression::parse(context)?;

		context.lexer.consume(Token::Colon)?;

		let body = context.parse_block(Statement::parse)?;

		let mut else_ifs = Vec::new();
		let mut else_block = Vec::new();
		while context.lexer.peek() == Some(Token::Else) {
			context.lexer.next();

			if context.lexer.peek() == Some(Token::If) {
				context.lexer.next();

				let condition = Expression::parse(context)?;

				context.lexer.consume(Token::Colon)?;
				let body = context.parse_block(Statement::parse)?;
				else_ifs.push(ElseIf { condition, body });
			} else {
				context.lexer.consume(Token::Colon)?;
				else_block = context.parse_block(Statement::parse)?;
				break;
			}
		}

		Ok(If { condition, body, else_ifs, else_block })
	}
}
