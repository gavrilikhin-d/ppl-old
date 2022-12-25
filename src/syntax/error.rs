use std::fmt::Display;

use thiserror::Error;
use miette::{SourceSpan, Diagnostic};

use super::Token;

/// Diagnostic for unwanted extra tokens
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
#[error("invalid token")]
#[diagnostic(code(lexer::invalid_token))]
pub struct InvalidToken {
	/// Span of the token
	#[label]
	pub at: SourceSpan
}

/// Diagnostic for unwanted extra tokens
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
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
#[derive(Error, Debug, Clone, Diagnostic, PartialEq, Eq)]
#[diagnostic(code(lexer::missing_token))]
pub struct MissingToken {
	/// Expected alternatives
	pub expected: Vec<Token>,

	/// Span of the token
	#[label("here")]
	pub at: SourceSpan
}

impl Display for MissingToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		debug_assert!(self.expected.len() > 0);

		if self.expected.len() == 1 {
			write!(f, "missing {}", self.expected[0])
		} else {
			write!(f, "missing tokens: ")?;
			self.expected
				.iter()
				.map(|token| format!("{}", token))
				.collect::<Vec<_>>()
				.join(" | ")
				.fmt(f)
		}
	}
}

/// Diagnostic for unexpected token
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
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

impl Display for UnexpectedToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		debug_assert!(self.expected.len() > 0);

		write!(
			f,
			"expected {}, got {}",
			self.expected
				.iter()
				.map(|token| format!("{}", token))
				.collect::<Vec<_>>()
				.join(" | "),
			self.got
		)
	}
}


/// Possible lexer errors
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
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
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
#[error("missing expression")]
#[diagnostic(
	code(parser::missing_expression),
)]
pub struct MissingExpression {
	/// Location, where expression was expected
	#[label("here")]
	pub at: SourceSpan
}

/// Diagnostic for missing declaration
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
#[error("missing declaration")]
#[diagnostic(
	code(parser::missing_declaration),
)]
pub struct MissingDeclaration {
	/// Location, where declaration was expected
	#[label("here")]
	pub at: SourceSpan
}

/// Diagnostic for missing statement
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
#[error("missing statement")]
#[diagnostic(
	code(parser::missing_statement),
)]
pub struct MissingStatement {
	/// Location, where statement was expected
	#[label("here")]
	pub at: SourceSpan
}

/// Diagnostic for missing variable's name
#[derive(Error, Diagnostic, Debug, Clone, PartialEq, Eq)]
#[error("missing variable's name")]
#[diagnostic(
	code(parser::missing_variable_name),
)]
pub struct MissingVariableName {
	/// Location, where name was expected
	#[label("variable's name is expected here")]
	pub at: SourceSpan
}

/// Possible parser errors
#[derive(Error, Diagnostic, Debug, PartialEq, Eq)]
pub enum ParseError {
	#[error(transparent)]
	#[diagnostic(transparent)]
	LexerError(#[from] LexerError),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingExpression(#[from] MissingExpression),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingDeclaration(#[from] MissingDeclaration),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingStatement(#[from] MissingStatement),
	#[error(transparent)]
	#[diagnostic(transparent)]
	MissingVariableName(#[from] MissingVariableName),
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
		Self::LexerError(err.into())
	}
}