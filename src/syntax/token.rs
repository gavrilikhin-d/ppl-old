use std::fmt::Display;

use logos::Logos;

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq, Eq, Clone)]
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

	/// Plus token
	#[token("+")]
	Plus,

	/// Minus token
	#[token("-")]
	Minus,

	/// Identifier
	#[regex("[_a-zA-Z][_a-zA-Z0-9]*")]
	Id,

	/// "let" token
	#[token("let")]
	Let,

	/// "mut" token
	#[token("mut")]
	Mut,

	/// "type" token
	#[token("type")]
	Type,

	/// '\n' token
	#[token("\n")]
	Newline,

	/// ':' token
	#[token(":")]
	Colon,

	/// '<' token
	#[token("<")]
	Less,

	/// '>' token
	#[token(">")]
	Greater,

	/// "fn" token
	#[token("fn")]
	Fn,

	/// "->" token
	#[token("->")]
	Arrow,

	/// String literal
	#[regex("\"[^\n\"]*\"")]
	String,

	/// '@' token
	#[token("@")]
	At,

	/// '(' token
	#[token("(")]
	LParen,

	/// ')' token
	#[token(")")]
	RParen,

	/// ',' comma
	#[token(",")]
	Comma,

	/// '\t' token
	#[token("\t")]
	Tab,

	/// Error token
	#[error]
	#[regex("[ ]+", logos::skip)]
	#[regex("//[^\n]*", logos::skip)]
	Error
}

impl Token {
	/// Check if token is an operator
	pub fn is_operator(&self) -> bool {
		match self {
			Token::Assign | Token::Plus | Token::Minus | Token::Less | Token::Greater => true,
			_ => false
		}
	}

	/// Check if token is a whitespace token
	pub fn is_whitespace(&self) -> bool {
		match self {
			Token::Newline => true,
			_ => false
		}
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::None => write!(f, "none"),
			Token::Assign => write!(f, "="),
			Token::Plus => write!(f, "+"),
			Token::Minus => write!(f, "-"),
			Token::Colon => write!(f, ":"),
			Token::Less => write!(f, "<"),
			Token::Greater => write!(f, ">"),
			Token::Fn => write!(f, "fn"),
			Token::Arrow => write!(f, "->"),
			Token::At => write!(f, "@"),
			Token::LParen => write!(f, "("),
			Token::RParen => write!(f, ")"),
			Token::Let => write!(f, "let"),
			Token::Mut => write!(f, "mut"),
			Token::Type => write!(f, "type"),
			_ => write!(f, "{:?}", self),
		}
	}
}