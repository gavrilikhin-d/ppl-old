use std::str::FromStr;
use crate::syntax::Token;
use logos::{Logos, Lexer};

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::error::*;

use crate::syntax::token::Consume;

use super::WithOffset;

use derive_more::From;


/// Trait for parsing using lexer
pub trait Parse where Self: Sized {
	type Err;

	/// Parse starting from current lexer state
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err>;
}

/// AST for compile time known values
#[derive(Debug, PartialEq, AST, Clone)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { offset: usize, value: String },
}

impl Parse for Literal {
	type Err = ParseError;

	/// Parse literal using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		let token = lexer.consume_one_of(&[Token::None, Token::Integer])?;

		match token {
			Token::None => Ok(Literal::None { offset: lexer.span().start }),
			Token::Integer => Ok(Literal::Integer { offset: lexer.span().start, value: lexer.slice().to_string() }),
			Token::Assign | Token::Id | Token::Let | Token::Error => unreachable!("consume_one_of returned unexpected token"),
		}
	}
}

/// AST for variable reference
#[derive(Debug, PartialEq, AST, Clone)]
pub struct VariableReference {
	/// Referenced variable name
	pub name: WithOffset<String>
}

impl Parse for VariableReference {
	type Err = ParseError;

	/// Parse variable reference using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		lexer.consume(Token::Id)?;

		let offset = lexer.span().start;
		let name = lexer.slice().to_string();

		Ok(VariableReference { name: WithOffset { offset, value: name }})
	}
}

/// Any PPL expression
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Expression {
	Literal(Literal),
	VariableReference(VariableReference),
}

impl Parse for Expression {
	type Err = ParseError;

	/// Parse expression using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		let mut copy = lexer.clone();
		let token = copy.consume_one_of(
			&[Token::None, Token::Integer, Token::Id]
		);
		if token.is_err() {
			return Err(
				MissingExpression {
					at: (copy.span().start - 1).into()
				}.into()
			)
		}

		match token.unwrap() {
			Token::None | Token::Integer => Ok(Expression::Literal(Literal::parse(lexer)?)),
			Token::Id => Ok(Expression::VariableReference(VariableReference::parse(lexer)?)),
			Token::Assign | Token::Let | Token::Error => unreachable!("consume_one_of returned unexpected token"),
		}
	}
}



/// Declaration of the variable
#[derive(Debug, PartialEq, AST, Clone)]
pub struct VariableDeclaration {
	/// Name of variable
	pub name: WithOffset<String>,
	/// Initializer for variable
	pub initializer: Expression,

	/// Is this variable mutable
	pub mutable: bool,
}

impl VariableDeclaration {
	/// Is this a mutable variable?
	pub fn is_mutable(&self) -> bool {
		self.mutable
	}

	/// Is this an immutable variable?
	pub fn is_immutable(&self) -> bool {
		!self.is_mutable()
	}
}

impl Parse for VariableDeclaration {
	type Err = ParseError;

	/// Parse variable declaration using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		lexer.consume(Token::Let)?;

		lexer.consume(Token::Id)?;

		let name = WithOffset {
			value: lexer.slice().to_string(),
			offset: lexer.span().start
		};

		lexer.consume(Token::Assign)?;

		Ok(
			VariableDeclaration {
				name: name,
				initializer: Expression::parse(lexer)?,
				mutable: false,
			}
		)
	}
}

/// Any PPL declaration
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Declaration {
	Variable(VariableDeclaration)
}

impl Parse for Declaration {
	type Err = ParseError;

	/// Parse declaration using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		VariableDeclaration::parse(lexer).map(Declaration::Variable)
	}
}

/// AST for assignment
#[derive(Debug, PartialEq, AST, Clone)]
pub struct Assignment {
	/// Target to assign to
	pub target: Expression,
	/// Expression to assign
	pub value: Expression,
}

impl Parse for Assignment {
	type Err = ParseError;

	/// Parse assignment using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		let target = Expression::parse(lexer)?;

		lexer.consume(Token::Assign)?;

		let value = Expression::parse(lexer)?;

		Ok(Assignment { target, value })
	}
}

/// Any PPL statement
#[derive(Debug, PartialEq, AST, Clone, From)]
pub enum Statement {
	Declaration(Declaration),
	Expression(Expression),
	Assignment(Assignment),
}

impl Parse for Statement {
	type Err = ParseError;

	/// Parse statement using lexer
	fn parse(lexer: &mut Lexer<Token>) -> Result<Self, Self::Err> {
		let mut copy = lexer.clone();
		let token = copy.consume_one_of(
			&[Token::None, Token::Integer, Token::Id, Token::Let]
		);
		if token.is_err() {
			return Err(token.unwrap_err().into())
		}

		match token.unwrap() {
			Token::Let => Declaration::parse(lexer).map(|decl| Statement::Declaration(decl)),
			Token::None | Token::Integer | Token::Id => {
				let target = Expression::parse(lexer)?;

				let mut copy = lexer.clone();
				if copy.next() != Some(Token::Assign) {
					return Ok(target.into())
				}

				// Skip '='
				lexer.next();

				Ok(Assignment {
					target,
					value: Expression::parse(lexer)?
				}.into())
			},
			Token::Assign | Token::Error => unreachable!("consume_one_of returned unexpected token"),
		}
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
fn test_variable_declaration() {
	let var = "let x = 1".parse::<VariableDeclaration>().unwrap();
	assert_eq!(
		var,
		VariableDeclaration {
			name: WithOffset { offset: 4, value: "x".to_string(), },
			initializer: Literal::Integer { offset: 8, value: "1".to_string() }.into()
		}
	);
}

#[test]
fn test_error() {
	let literal = "123a".parse::<Literal>();
	assert!(literal.is_err());
}