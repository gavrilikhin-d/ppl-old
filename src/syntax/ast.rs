use std::str::FromStr;
use crate::syntax::Token;
use logos::{Logos, Lexer};

/// Trait for lexer to consume tokens
trait Consume {
	type Err;

	/// Parse next token and check that it has specified type
	fn consume(&mut self, token: Token) -> Result<(), Self::Err>;
}

impl<'source> Consume for logos::Lexer<'source, Token> {
	type Err = ();

	fn consume(&mut self, token: Token) -> Result<(), Self::Err> {
		if self.next() != Some(token) {
			return Err(())
		}

		Ok(())
	}
}

/// Trait for parsing using lexer
trait Parse where Self: Sized {
	type Err;

	/// Parse starting from current lexer state
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err>;
}

/// AST for compile time known values
#[derive(Debug, PartialEq)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { offset: usize, value: String },
}

impl Parse for Literal {
	type Err = ();

	/// Parse literal using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		let token = lexer.next().unwrap();

		match token {
			Token::None => Ok(Literal::None { offset: lexer.span().start }),
			Token::Integer => Ok(Literal::Integer { offset: lexer.span().start, value: lexer.slice().to_string() }),
			Token::Assign | Token::Id | Token::Let | Token::Error => Err(()),
		}
	}
}

impl FromStr for Literal {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lexer = Token::lexer(s);

		let res = Literal::parse(&mut lexer);

		if lexer.next() != None {
			return Err(());
		}
		res
	}
}

/// Value at some offset
#[derive(Debug, PartialEq)]
pub struct WithOffset<T> {
	/// Offset of the value
	pub offset: usize,
	/// Value at some offset
	pub value: T,
}

/// Declaration of the variable
#[derive(Debug, PartialEq)]
pub struct VariableDeclaration {
	/// Name of variable
	pub name: WithOffset<String>,
	/// Initializer for variable
	pub initializer: Literal,
}

impl Parse for VariableDeclaration {
	type Err = ();

	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		lexer.consume(Token::Let)?;

		lexer.consume(Token::Id)?;

		let name = WithOffset {
			value: lexer.slice().to_string(),
			offset: lexer.span().start
		};

		lexer.consume(Token::Assign)?;

		Ok(VariableDeclaration { name: name, initializer: Literal::parse( lexer)? })
	}
}

impl FromStr for VariableDeclaration {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lexer = Token::lexer(s);

		let res = Self::parse(&mut lexer);

		if lexer.next() != None {
			return Err(());
		}
		res
	}
}

#[test]
fn test_none() {
	let literal = "none".parse::<Literal>().unwrap();
	assert_eq!(literal, Literal::None { offset: 0 });
}

#[test]
fn test_integer() {
	let literal = "123".parse::<Literal>().unwrap();
	assert_eq!(literal, Literal::Integer { offset: 0, value: "123".to_string() });
}

#[test]
fn test_variable_declaration() {
	let var = "let x = 1".parse::<VariableDeclaration>().unwrap();
	assert_eq!(
		var,
		VariableDeclaration {
			name: WithOffset { offset: 4, value: "x".to_string(), },
			initializer: Literal::Integer { offset: 8, value: "1".to_string() }
		}
	);
}

#[test]
fn test_error() {
	let literal = "123a".parse::<Literal>();
	assert!(literal.is_err());
}