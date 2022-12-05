use std::fmt::Display;

use logos::Logos;

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
	#[regex("[ ]+", logos::skip)]
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