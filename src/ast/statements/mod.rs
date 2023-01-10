mod assignment;
pub use assignment::*;

mod ret;
pub use ret::*;

extern crate ast_derive;
use ast_derive::AST;

use crate::ast::{Declaration, Expression};
use crate::syntax::error::MissingStatement;
use crate::syntax::StartsHere;
use crate::syntax::{error::ParseError, Lexer, Parse, Token};

use derive_more::From;

use super::Annotation;

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone, From)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
	Return(Return)
}

impl StartsHere for Statement {
    /// Check that statement may start at current lexer position
    fn starts_here(lexer: &mut impl Lexer) -> bool {
        Annotation::starts_here(lexer)
            || Declaration::starts_here(lexer)
            || Expression::starts_here(lexer)
            || Assignment::starts_here(lexer)
			|| Return::starts_here(lexer)
    }
}

impl Parse for Statement {
    type Err = ParseError;

    /// Parse statement using lexer
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        lexer.skip_spaces();

        if !Statement::starts_here(lexer) {
            return Err(MissingStatement {
                at: lexer.span().end.into(),
            }
            .into());
        }

        let mut annotations = Vec::new();
        while Annotation::starts_here(lexer) {
            annotations.push(Annotation::parse(lexer)?);
            lexer.skip_spaces();
        }



        let mut res: Statement = if Declaration::starts_here(lexer) {
			Declaration::parse(lexer)?.into()
		}
		else if Expression::starts_here(lexer) {
			let target = Expression::parse(lexer)?;

			if lexer.consume(Token::Assign).is_err() {
				target.into()
			} else {
				Assignment {
					target,
					value: Expression::parse(lexer)?,
				}
				.into()
			}
		}
		else {
			match lexer.peek() {
				Some(Token::Return) => Return::parse(lexer)?.into(),
				t => unreachable!(
					"Unexpected token {:#?} at start of statement", t
				),
			}
		};

        if !annotations.is_empty() {
            if let Statement::Declaration(Declaration::Function(ref mut decl)) = res {
                decl.annotations = annotations;
            } else {
                unimplemented!("Annotations are not supported for this statement");
            }
        }

        if lexer.peek().is_some() {
            lexer.consume(Token::Newline)?;
        }

        Ok(res)
    }
}
