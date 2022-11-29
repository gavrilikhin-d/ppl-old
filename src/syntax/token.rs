use logos::Logos;

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq)]
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

	/// Error token
	#[error]
	Error
}