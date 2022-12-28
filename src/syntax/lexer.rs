use logos::{Logos, Span};

use crate::syntax::error::{LexerError, InvalidToken, UnexpectedToken, MissingToken};

use super::{Token, StringWithOffset};

/// Lexer for PPL
#[derive(Debug, Clone)]
pub struct Lexer<'source> {
	/// Logos lexer for tokens
    lexer: logos::Lexer<'source, Token>,
	/// Span of current token
	span: Span,
	/// Peeked token
    peeked: Option<Token>,
	/// Current indentation level
	indentation: usize,
}

impl<'source> Lexer<'source> {
	/// Create new lexer
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// ```
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
			span: 0..0,
            peeked: None,
			indentation: 0,
        }
    }

	/// Peek next token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(lexer.peek(), Some(Token::Integer));
	/// assert_eq!(lexer.span(), 0..0);
	///
	/// assert_eq!(lexer.next(), Some(Token::Integer));
	/// assert_eq!(lexer.span(), 0..2);
	/// ```
    pub fn peek(&mut self) -> Option<Token> {
        if self.peeked.is_none() {
			self.peeked = self.lexer.next();
        }
		self.peeked.clone()
    }

	/// Get span of next token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(lexer.peek_span(), 0..2);
	/// assert_eq!(lexer.span(), 0..0);
	/// ```
	pub fn peek_span(&mut self) -> Span {
		if self.peeked.is_none() {
			self.peek();
		}
		self.lexer.span()
	}

	/// Get slice of source code for next token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(lexer.peek_slice(), "42");
	/// assert_eq!(lexer.span(), 0..0);
	/// ```
	pub fn peek_slice(&mut self) -> &'source str {
		if self.peeked.is_none() {
			self.peek();
		}
		self.lexer.slice()
	}

	/// Get span of current token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(lexer.next(), Some(Token::Integer));
	/// assert_eq!(lexer.span(), 0..2);
	/// ```
	pub fn span(&self) -> Span {
		self.span.clone()
	}

	/// Get slice of current token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.slice(), "");
	/// assert_eq!(lexer.next(), Some(Token::Integer));
	/// assert_eq!(lexer.slice(), "42");
	/// ```
	pub fn slice(&self) -> &'source str {
		&self.lexer.source()[self.span.clone()]
	}

	/// Get string with offset of current token
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer, StringWithOffset};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.next(), Some(Token::Integer));
	/// assert_eq!(
	/// 	lexer.string_with_offset(),
	/// 	StringWithOffset::from("42").at(0)
	/// );
	pub fn string_with_offset(&self) -> StringWithOffset {
		StringWithOffset::from(self.slice()).at(self.span().start)
	}

	/// Bumps the end of currently lexed token by `n` bytes.
    ///
    /// # Panics
    ///
    /// Panics if adding `n` to current offset would place the `Lexer` beyond the last byte,
    /// or in the middle of an UTF-8 code point (does not apply when lexing raw `&[u8]`).
	pub fn bump(&mut self, n: usize) {
		self.span.end += n;
		self.peeked.take();
		self.lexer.bump(n)
	}
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
		self.peek();
		self.span = self.lexer.span();
		self.peeked.take()
    }
}

impl<'source> Lexer<'source> {
	/// Try match next token with given type
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer, error::*};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.try_match(Token::Integer), Ok(()));
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(
	/// 	lexer.try_match(Token::Id),
	/// 	Err(
	/// 		UnexpectedToken {
	/// 			expected: vec![Token::Id],
	/// 			got: Token::Integer,
	/// 			at: lexer.peek_span().into()
	/// 		}.into()
	/// 	)
	/// );
	/// ```
	pub fn try_match(&mut self, token: Token) -> Result<(), LexerError> {
		self.try_match_one_of(&[token]).map(|_| ())
	}

	/// Try match next token with one of specified types
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer, error::*};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(lexer.try_match_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
	/// assert_eq!(lexer.span(), 0..0);
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.span(), 0..0);
	/// assert_eq!(
	/// 	lexer.try_match_one_of(&[Token::None, Token::Id]),
	/// 	Err(
	/// 		UnexpectedToken {
	/// 			expected: vec![Token::None, Token::Id],
	/// 			got: Token::Integer,
	/// 			at: lexer.peek_span().into()
	/// 		}.into()
	/// 	)
	/// );
	/// assert_eq!(lexer.span(), 0..0);
	/// ```
	pub fn try_match_one_of(&mut self, tokens: &[Token]) -> Result<Token, LexerError> {
		debug_assert!(tokens.len() > 0);

		let token = self.peek();
		if token.is_none() {
			return Err(MissingToken {
				expected: tokens.to_vec(),
				at: self.span().end.into()
			}.into());
		}

		let token = token.unwrap();

		if !tokens.contains(&token) {
			if token == Token::Error {
				return Err(InvalidToken{at: self.peek_span().into()}.into());
			}

			return Err(UnexpectedToken {
				expected: tokens.to_owned(),
				got: token,
				at: self.peek_span().into()
			}.into());
		}

		Ok(token)
	}

	/// Lex next token if it has specified type
	///
	/// **Note:** doesn't lex, on failure
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer, error::*};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.consume(Token::Integer), Ok("42".into()));
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(
	/// 	lexer.consume(Token::Id),
	/// 	Err(
	/// 		UnexpectedToken {
	/// 			expected: vec![Token::Id],
	/// 			got: Token::Integer,
	/// 			at: lexer.peek_span().into()
	/// 		}.into()
	/// 	)
	/// );
	/// ```
	pub fn consume(&mut self, token: Token) -> Result<StringWithOffset, LexerError> {
		self.consume_one_of(&[token]).map(|_| self.string_with_offset())
	}

	/// Lex next token if it has one of specified types
	///
	/// **Note:** doesn't lex, on failure
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer, error::*};
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(lexer.consume_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
	///
	/// let mut lexer = Lexer::new("42");
	/// assert_eq!(
	/// 	lexer.consume_one_of(&[Token::None, Token::Id]),
	/// 	Err(
	/// 		UnexpectedToken {
	/// 			expected: vec![Token::None, Token::Id],
	/// 			got: Token::Integer,
	/// 			at: lexer.peek_span().into()
	/// 		}.into()
	/// 	)
	/// );
	/// ```
	pub fn consume_one_of(&mut self, tokens: &[Token]) -> Result<Token, LexerError> {
		let token = self.try_match_one_of(tokens)?;
		self.next();
		Ok(token)
	}
}

impl<'source> Lexer<'source> {
	/// Skip space tokens
	///
	/// # Example
	/// ```
	/// use ppl::syntax::{Token, Lexer};
	///
	/// let mut lexer = Lexer::new("\n  \n42");
	/// assert_eq!(lexer.peek(), Some(Token::Newline));
	/// lexer.skip_spaces();
	/// assert_eq!(lexer.peek(), Some(Token::Integer));
	/// ```
	pub fn skip_spaces(&mut self) -> &mut Self {
		while self.peek().map_or(false, |token| token.is_whitespace()) {
			self.next();
		}
		self
	}
}

impl<'source> Lexer<'source> {
	/// Get current indentation level
	pub fn indentation(&self) -> usize {
		self.indentation
	}

	/// Skip indentation.
	/// Changes current indentation level to the amount of tabs skipped
	pub fn skip_indentation(&mut self) -> &mut Self {
		self.indentation = 0;
		while self.peek() == Some(Token::Tab) {
			self.next();
			self.indentation += 1;
		}
		self
	}
}