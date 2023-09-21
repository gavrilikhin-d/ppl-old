extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Context, Lexer, Parse, Ranged, StartsHere, Token};

use super::Expression;

/// AST for tuple
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Tuple {
    /// Offset of '('
    pub lparen: usize,
    /// Expression in parentheses
    pub expressions: Vec<Expression>,
    /// Offset of ')'
    pub rparen: usize,
}

impl Ranged for Tuple {
    fn start(&self) -> usize {
        self.lparen
    }

    fn end(&self) -> usize {
        self.rparen + 1
    }
}

impl StartsHere for Tuple {
    /// Check that tuple may start at current lexer position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.peek() == Some(Token::LParen)
    }
}

impl Parse for Tuple {
    type Err = ParseError;

    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let lparen = context.lexer.consume(Token::LParen)?.start();

        let mut expressions = Vec::new();
        while context.lexer.peek().map_or(false, |t| t != Token::RParen) {
            expressions.push(Expression::parse(context)?);

            if context.lexer.peek().map_or(true, |t| t != Token::Comma) {
                break;
            }

            context.lexer.consume(Token::Comma)?;
        }

        let rparen = context.lexer.consume(Token::RParen)?.start();

        Ok(Tuple {
            lparen,
            expressions,
            rparen,
        })
    }
}
