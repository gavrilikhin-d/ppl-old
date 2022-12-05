use std::fmt::Display;

use logos::Logos;

use crate::syntax::error::{LexerError, InvalidToken, UnexpectedToken, MissingToken};

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token
{
	/// None literal
	#[token("none")]
	None,

	/// Integer literal
	#[regex("[0-9]+")]
	Integer,

	/// Assign token
	#[token("=")]
	Assign,

	/// Identifier
	#[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
	Id,

	/// "let" token
	#[token("let")]
	Let,

	/// Error token
	#[error]
	#[regex("[ \n]+", logos::skip)]
	Error
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::None => write!(f, "'none'"),
			Token::Assign => write!(f, "'='"),
			Token::Let => write!(f, "'let'"),
			_ => write!(f, "{:?}", self),
		}
	}
}

/// Trait for lexer to consume tokens
pub trait Consume {
	type Err;

	/// Lex next token and check that it has specified type
	fn consume(&mut self, token: Token) -> Result<(), Self::Err> {
		self.consume_one_of(&[token]).map(|_| ())
	}

	/// Lex next token and check that it has one of the specified types
	fn consume_one_of(&mut self, tokens: &[Token]) -> Result<Token, Self::Err>;
}


impl<'source> Consume for logos::Lexer<'source, Token> {
	type Err = LexerError;

	/// Parse next token and check that it has specified type
	///
	/// # Example
	/// ```
	/// use ppl::syntax::Token;
	/// use ppl::syntax::{ast::Consume, error::*};
	/// use logos::Logos;
	///
	/// let mut lexer = ppl::syntax::Token::lexer("42");
	/// assert_eq!(lexer.consume(Token::Integer), Ok(()));
	///
	/// let mut lexer = ppl::syntax::Token::lexer("42");
	/// assert_eq!(
	/// 	lexer.consume(Token::Id),
	/// 	Err(
	/// 		UnexpectedToken {
	/// 			expected: vec![Token::Id],
	/// 			got: Token::Integer,
	/// 			at: lexer.span().into()
	/// 		}.into()
	/// 	)
	/// );
	/// ```
	fn consume_one_of(&mut self, tokens: &[Token]) -> Result<Token, Self::Err> {
		debug_assert!(tokens.len() > 0);

		let token = self.next();
		if token.is_none() {
			return Err(MissingToken {
				expected: tokens.to_vec(),
				at: (self.span().end - 1).into()
			}.into());
		}

		let token = token.unwrap();

		if !tokens.contains(&token) {
			if token == Token::Error {
				return Err(InvalidToken{at: self.span().into()}.into());
			}

			return Err(UnexpectedToken {
				expected: tokens.to_owned(),
				got: token,
				at: self.span().into()
			}.into());
		}

		Ok(token)
	}
}