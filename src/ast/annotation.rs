extern crate ast_derive;

use ast_derive::AST;

use crate::syntax::{
    error::ParseError, Context, Lexer, Parse, StartsHere, StringWithOffset, Token,
};

use super::Expression;

/// Annotations for statements
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Annotation {
    /// Annotation name
    pub name: StringWithOffset,
    /// Arguments of annotation
    pub args: Vec<Expression>,
}

impl StartsHere for Annotation {
    /// Check if annotation 100% starts at current position
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context.lexer.try_match(Token::At).is_ok()
    }
}

impl Parse for Annotation {
    type Err = ParseError;

    /// Parse annotation using lexer
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        context.lexer.consume(Token::At)?;

        let name = context.lexer.consume(Token::Id)?;
        let mut args = Vec::new();
        if context.lexer.consume(Token::LParen).is_ok() {
            while context.lexer.peek() != Some(Token::RParen) {
                args.push(Expression::parse(context)?);
                if context.lexer.peek() != Some(Token::Colon) {
                    break;
                }

                context.lexer.consume(Token::Comma)?;
            }
            context.lexer.consume(Token::RParen)?;
        }

        Ok(Annotation { name, args })
    }
}
