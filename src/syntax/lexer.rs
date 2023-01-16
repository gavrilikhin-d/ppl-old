use std::{cell::RefCell, io::Write};

use logos::{Logos, Span};

use crate::syntax::error::{InvalidToken, LexerError, MissingToken, UnexpectedToken};

use super::{StringWithOffset, Token};

/// Trait for PPL's lexers
pub trait Lexer: Iterator<Item = Token> {
    /// Get source code of lexer
    fn source(&self) -> &str;

	/// Get current token
	fn token(&self) -> Option<Token>;

    /// Peek next token
    fn peek(&self) -> Option<Token>;

    /// Get span of peeked token
    fn peek_span(&self) -> Span;

    /// Get slice of peeked token
    fn peek_slice(&self) -> &str;

    /// Get span of current token
    fn span(&self) -> Span;

    /// Get slice of current token
    fn slice(&self) -> &str;

    /// Get string with offset of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, StringWithOffset};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(
    /// 	lexer.string_with_offset(),
    /// 	StringWithOffset::from("42").at(0)
    /// );
    /// ```
    fn string_with_offset(&self) -> StringWithOffset {
        StringWithOffset::from(self.slice()).at(self.span().start)
    }

    /// Try match next token with given type
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.try_match(Token::Integer), Ok(()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
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
    fn try_match(&self, token: Token) -> Result<(), LexerError> {
        self.try_match_one_of(&[token]).map(|_| ())
    }

    /// Try match next token with one of specified types
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.try_match_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
    /// assert_eq!(lexer.span(), 0..0);
    ///
    /// let mut lexer = FullSourceLexer::new("42");
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
    fn try_match_one_of(&self, tokens: &[Token]) -> Result<Token, LexerError> {
        debug_assert!(tokens.len() > 0);

        let token = self.peek();
        if token.is_none() {
            return Err(MissingToken {
                expected: tokens.to_vec(),
                at: self.span().end.into(),
            }
            .into());
        }

        let token = token.unwrap();

        if !tokens.contains(&token) {
            if token == Token::Error {
                return Err(InvalidToken {
                    at: self.peek_span().into(),
                }
                .into());
            }

            return Err(UnexpectedToken {
                expected: tokens.to_owned(),
                got: token,
                at: self.peek_span().into(),
            }
            .into());
        }

        Ok(token)
    }

    /// Lex next token if it has specified type
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.consume(Token::Integer), Ok("42".into()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
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
    fn consume(&mut self, token: Token) -> Result<StringWithOffset, LexerError> {
        self.consume_one_of(&[token])
            .map(|_| self.string_with_offset())
    }

    /// Lex next token if it has one of specified types
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.consume_one_of(&[Token::Integer, Token::Id]), Ok(Token::Integer));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
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
    fn consume_one_of(&mut self, tokens: &[Token]) -> Result<Token, LexerError> {
        let token = self.try_match_one_of(tokens)?;
        self.next();
        Ok(token)
    }

	/// Lex next token if it's an operator
    ///
    /// **Note:** doesn't lex, on failure
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer, error::*};
    ///
    /// let mut lexer = FullSourceLexer::new("+");
    /// assert_eq!(lexer.consume_operator(), Ok("+".into()));
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(
    /// 	lexer.consume_operator(),
    /// 	Err(
    /// 		UnexpectedToken {
    /// 			expected: vec![
	/// 				Token::Operator, Token::Less, Token::Greater
	/// 			],
    /// 			got: Token::Integer,
    /// 			at: lexer.peek_span().into()
    /// 		}.into()
    /// 	)
    /// );
    /// ```
	fn consume_operator(&mut self) -> Result<StringWithOffset, LexerError> {
		self.consume_one_of(
			&[Token::Operator, Token::Less, Token::Greater]
		)?;
		Ok(self.string_with_offset())
	}

    /// Skip space tokens
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("\n  \n42");
    /// assert_eq!(lexer.peek(), Some(Token::Newline));
    /// lexer.skip_spaces();
    /// assert_eq!(lexer.peek(), Some(Token::Integer));
    /// ```
    fn skip_spaces(&mut self) -> &mut Self {
        while self.peek().map_or(false, |token| token.is_whitespace()) {
            self.next();
        }
        self
    }

    /// Get current indentation level
    fn indentation(&self) -> usize;

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self;
}

/// Lexer for full source code of PPL
pub struct FullSourceLexer<'source> {
    /// Logos lexer for tokens
    lexer: RefCell<logos::Lexer<'source, Token>>,
    /// Span of current token
    span: Span,
	/// Current token
	token: Option<Token>,
    /// Peeked token
    peeked: RefCell<Option<Token>>,
    /// Current indentation level
    indentation: usize,
}

impl<'source> FullSourceLexer<'source> {
    /// Create new lexer
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source).into(),
            span: 0..0,
			token: None,
            peeked: None.into(),
            indentation: 0,
        }
    }
}

impl<'source> Iterator for FullSourceLexer<'source> {
    type Item = Token;

    /// Lex next token
    fn next(&mut self) -> Option<Token> {
        if self.peek() == Some(Token::Newline) {
			self.indentation = 0;
		}
        self.span = self.lexer.get_mut().span();
        self.token = self.peeked.take();
		self.token()
    }
}

impl Lexer for FullSourceLexer<'_> {
	/// Get current token
	fn token(&self) -> Option<Token> {
		self.token.clone()
	}

    /// Get source code of lexer
    fn source(&self) -> &str {
        self.lexer.borrow().source()
    }

    /// Peek next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..0);
    ///
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn peek(&self) -> Option<Token> {
        if self.peeked.borrow().is_none() {
            *self.peeked.borrow_mut() = self.lexer.borrow_mut().next();
			if self.token == Some(Token::Newline) {
				while *self.peeked.borrow() == Some(Token::Newline) {
					*self.peeked.borrow_mut() = self.lexer.borrow_mut().next();
				}
			}
        }
        self.peeked.borrow().clone()
    }

    /// Get span of next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_span(), 0..2);
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_span(&self) -> Span {
        self.peek();
        self.lexer.borrow_mut().span()
    }

    /// Get slice of source code for next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_slice(), "42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_slice(&self) -> &str {
        self.peek();
        self.lexer.borrow_mut().slice()
    }

    /// Get span of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn span(&self) -> Span {
        self.span.clone()
    }

    /// Get slice of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.slice(), "");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.slice(), "42");
    /// ```
    fn slice(&self) -> &str {
        &self.lexer.borrow_mut().source()[self.span.clone()]
    }

    /// Get current indentation level
    fn indentation(&self) -> usize {
        self.indentation
    }

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self {
        while self.peek() == Some(Token::Tab) {
            self.next();
            self.indentation += 1;
        }
        self
    }
}

/// Lexer for reading from interactive stream (stdin)
pub struct InteractiveLexer {
	/// Prompt to display before reading next line
	pub prompt: String,
	/// Used to override next prompt
	next_prompt: RefCell<Option<String>>,
    /// Current source code of lexer
    source: RefCell<String>,
    /// Span of current token
    span: Span,
	/// Current token
	token: Option<Token>,
    /// Current indentation level
    indentation: usize,
}

impl InteractiveLexer {
    /// Create new interactive lexer
    pub fn new() -> Self {
        Self {
			prompt: "... ".to_string(),
			next_prompt: None.into(),
            source: String::new().into(),
            span: 0..0,
			token: None,
            indentation: 0,
        }
    }

    /// Create lexer and set it state at the end of current token
    fn lexer<'s>(&'s self) -> logos::Lexer<'_, Token> {
        let mut lexer = Token::lexer(self.source());
        lexer.bump(self.span.end);
        lexer
    }

    /// Request next line
    fn request_line(&self) {
		if let Some(prompt) = self.next_prompt.take() {
        	print!("{}", prompt);
		}
		else
		{
			print!("{}", self.prompt);
		}
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        self.source.borrow_mut().push_str(&line);
    }

    /// Request next line if lexer is at the end of source code
    fn maybe_request_line(&self) {
        if self.span.end == self.source.borrow().len() {
            self.request_line()
        }
    }

	/// Override next prompt
	pub fn override_next_prompt(&mut self, prompt: &str) {
		self.next_prompt = Some(prompt.to_string()).into()
	}
}

impl Iterator for InteractiveLexer {
    type Item = Token;

    /// Lex next token
    fn next(&mut self) -> Option<Token> {
        self.maybe_request_line();
		let mut lexer = self.lexer();
        let mut peeked = lexer.next();
		if self.token == Some(Token::Newline) {
			while peeked == Some(Token::Newline) {
				peeked = lexer.next();
			}
		}
		self.span = lexer.span();
		self.token = peeked;
		if self.token == Some(Token::Newline) {
			self.indentation = 0;
		}
        self.token()
    }
}

impl Lexer for InteractiveLexer {
    /// Get source code of lexer
    fn source(&self) -> &str {
        unsafe { &*self.source.as_ptr() }
    }

	/// Get current token
	fn token(&self) -> Option<Token> {
		self.token.clone()
	}

    /// Peek next token
    fn peek(&self) -> Option<Token> {
        self.maybe_request_line();
		let mut lexer = self.lexer();
        let mut peeked = lexer.next();
		if self.token == Some(Token::Newline) {
			while peeked == Some(Token::Newline) {
				peeked = lexer.next();
			}
		}
		peeked
    }

    /// Get span of next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_span(), 0..2);
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_span(&self) -> Span {
        self.maybe_request_line();
        let mut lexer = self.lexer();
        lexer.next();
        lexer.span()
    }

    /// Get slice of source code for next token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.peek_slice(), "42");
    /// assert_eq!(lexer.span(), 0..0);
    /// ```
    fn peek_slice(&self) -> &str {
        self.maybe_request_line();
        let mut lexer = self.lexer();
        lexer.next();
        lexer.slice()
    }

    /// Get span of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.span(), 0..0);
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.span(), 0..2);
    /// ```
    fn span(&self) -> Span {
        self.span.clone()
    }

    /// Get slice of current token
    ///
    /// # Example
    /// ```
    /// use ppl::syntax::{Token, Lexer, FullSourceLexer};
    ///
    /// let mut lexer = FullSourceLexer::new("42");
    /// assert_eq!(lexer.slice(), "");
    /// assert_eq!(lexer.next(), Some(Token::Integer));
    /// assert_eq!(lexer.slice(), "42");
    /// ```
    fn slice(&self) -> &str {
        self.source()[self.span()].into()
    }

    /// Get current indentation level
    fn indentation(&self) -> usize {
        self.indentation
    }

    /// Skip indentation.
    /// Changes current indentation level to the amount of tabs skipped
    fn skip_indentation(&mut self) -> &mut Self {
        while self.peek() == Some(Token::Tab) {
            self.next();
            self.indentation += 1;
        }
        self
    }
}
