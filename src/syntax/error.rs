use std::fmt::Display;

use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

use super::Token;

/// Diagnostic for unwanted extra tokens
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("invalid token")]
#[diagnostic(code(lexer::invalid_token))]
pub struct InvalidToken {
	/// Span of the token
	#[label]
	pub at: SourceSpan
}

/// Diagnostic for unwanted extra tokens
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("extra {token:?}")]
#[diagnostic(code(lexer::extra_token))]
pub struct ExtraToken {
	/// Extra token
	pub token: Token,

	/// Span of the token
	#[label]
	pub at: SourceSpan
}

/// Diagnostic for missing token
#[derive(Error, Debug, Clone, Diagnostic, PartialEq)]
#[diagnostic(code(lexer::missing_token))]
pub struct MissingToken {
	/// Expected alternatives
	pub expected: Vec<Token>,

	/// Span of the token
	#[label]
	pub at: SourceSpan
}

impl Display for MissingToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		debug_assert!(self.expected.len() > 0);

		if self.expected.len() == 1 {
			write!(f, "missing {:?}", self.expected[0])
		} else {
			write!(f, "missing tokens: ")?;
			self.expected
				.iter()
				.map(|token| format!("{:?}", token))
				.collect::<Vec<_>>()
				.join(" | ")
				.fmt(f)
		}
	}
}

/// Diagnostic for unexpected token
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("unexpected {got:?}")]
#[diagnostic(code(lexer::unexpected_token))]
pub struct UnexpectedToken {
	/// Expected alternatives
	pub expected: Vec<Token>,
	/// Actual token
	pub got: Token,

	/// Span of the token
	#[label]
	pub at: SourceSpan
}

impl UnexpectedToken {
	fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        debug_assert!(self.expected.len() > 0);

		Some(
			Box::new(
				if self.expected.len() == 1 {
					format!("expected {:?}", self.expected[0])
				} else {
					format!(
						"expected tokens: {}",
						self.expected
							.iter()
							.map(|token| format!("{:?}", token))
							.collect::<Vec<_>>()
							.join(" | ")
					)
				}
			)
		)
    }
}


/// Possible lexer errors
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
pub enum LexerError {
	#[error(transparent)]
	#[diagnostic(transparent)]
	InvalidToken(#[from] InvalidToken),
	#[error(transparent)]
	#[diagnostic(transparent)]
	ExtraToken(#[from] ExtraToken),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingToken(#[from] MissingToken),
	#[error(transparent)]
	#[diagnostic(transparent)]
	UnexpectedToken(#[from] UnexpectedToken),
}

/// Diagnostic for missing expressions
#[derive(Error, Diagnostic, Debug, Clone, PartialEq)]
#[error("missing expression")]
#[diagnostic(
	code(parser::missing_expression),
)]
pub struct MissingExpression {
	/// Location, where expression was expected
	#[label("here")]
	pub at: SourceSpan
}

/// Possible parser errors
#[derive(Error, Diagnostic, Debug, PartialEq)]
pub enum ParseError {
	#[error(transparent)]
	#[diagnostic(transparent)]
	LexerError(#[from] LexerError),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingExpression(#[from] MissingExpression)
}

impl From<InvalidToken> for ParseError {
	fn from(error: InvalidToken) -> Self {
		Self::LexerError(error.into())
	}
}

impl From<ExtraToken> for ParseError {
	fn from(error: ExtraToken) -> Self {
		Self::LexerError(error.into())
	}
}

impl From<MissingToken> for ParseError {
	fn from(error: MissingToken) -> Self {
		Self::LexerError(error.into())
	}
}

impl From<UnexpectedToken> for ParseError {
	fn from(err: UnexpectedToken) -> Self {
		ParseError::LexerError(err.into())
	}
}