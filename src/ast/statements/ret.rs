extern crate ast_derive;
use ast_derive::AST;

use crate::ast::Expression;
use crate::syntax::{StartsHere, Ranged, Context};
use crate::syntax::{error::ParseError, Lexer, Parse, Token};

/// AST for return statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Return {
	/// Offset for start of "return" keyword
	pub offset: usize,
    /// Returned value
    pub value: Option<Expression>,
}

impl Ranged for Return {
	fn start(&self) -> usize {
		self.offset
	}

	fn end(&self) -> usize {
		if self.value.is_some() {
			self.value.as_ref().unwrap().end()
		} else {
			self.offset + "return".len()
		}
	}
}

impl StartsHere for Return {
	/// Check that return may start at current lexer position
	fn starts_here(context: &mut Context<impl Lexer>) -> bool {
		context.lexer.peek() == Some(Token::Return)
	}
}

impl Parse for Return {
	type Err = ParseError;

	/// Parse return using lexer
	fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
		let offset = context.lexer.consume(Token::Return)?.start();

		let value = if Expression::starts_here(context) {
			Some(Expression::parse(context)?)
		} else {
			None
		};

		Ok(Return { offset, value })
	}
}