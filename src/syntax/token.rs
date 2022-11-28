use logos::Logos;

/// The different kinds of tokens that can be lexed.
#[derive(Logos, Debug, PartialEq)]
pub enum Token
{
	/// Integer literal
	#[regex("[0-9]+")]
	Integer,

	/// Error token
	#[error]
	Error
}

#[test]
fn test_integer() {
	let mut lexer = Token::lexer("123");
	assert_eq!(lexer.next(), Some(Token::Integer));
	assert_eq!(lexer.span(), 0..3);
	assert_eq!(lexer.slice(), "123");

	assert_eq!(lexer.next(), None);
}

#[test]
fn test_error() {
	let mut lexer = Token::lexer("123a");
	assert_eq!(lexer.next(), Some(Token::Integer));
	assert_eq!(lexer.span(), 0..3);
	assert_eq!(lexer.slice(), "123");

	assert_eq!(lexer.next(), Some(Token::Error));
	assert_eq!(lexer.span(), 3..4);
	assert_eq!(lexer.slice(), "a");

	assert_eq!(lexer.next(), None);
}