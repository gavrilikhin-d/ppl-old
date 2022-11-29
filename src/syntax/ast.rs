use std::str::FromStr;
use crate::syntax::Token;
use logos::Logos;

/// AST for compile time known values
#[derive(Debug, PartialEq)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { offset: usize, value: String },
}


impl FromStr for Literal {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lexer = Token::lexer(s);
		let token = lexer.next().unwrap();

		let res = match token {
			Token::None => Ok(Literal::None { offset: lexer.span().start }),
			Token::Integer => Ok(Literal::Integer { offset: lexer.span().start, value: lexer.slice().to_string() }),
			Token::Assign | Token::Id | Token::Error => Err(()),
		};

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
fn test_error() {
	let literal = "123a".parse::<Literal>();
	assert!(literal.is_err());
}