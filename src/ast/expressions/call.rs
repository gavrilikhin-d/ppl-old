extern crate ast_derive;

use ast_derive::AST;

use derive_more::{From, TryInto};

use super::Expression;

use crate::syntax::{error::ParseError, Lexer, Parse, Ranged, StartsHere, StringWithOffset, Token};

/// Cell of function
#[derive(Debug, PartialEq, Eq, AST, Clone, From, TryInto)]
pub enum CallNamePart {
    Text(StringWithOffset),
    Argument(Expression),
}

impl Ranged for CallNamePart {
    /// Get range of function call cell
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            CallNamePart::Text(text) => text.range(),
            CallNamePart::Argument(arg) => arg.range(),
        }
    }
}

impl Parse for CallNamePart {
    type Err = ParseError;

    /// Parse function call cell using lexer
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        let token = lexer.consume(Token::Id);
        Ok(if let Ok(text) = token {
            text.into()
        } else {
            Expression::parse(lexer)?.into()
        })
    }
}

/// AST for function call
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Call {
    /// Name parts of function call
    pub name_parts: Vec<CallNamePart>,
}

impl Call {
    /// Get name format of called function
    pub fn name_format(&self) -> String {
        let mut format = String::new();
        for (i, part) in self.name_parts.iter().enumerate() {
            if i > 0 {
                format.push(' ');
            }

            match part {
                CallNamePart::Text(text) => format.push_str(&text),
                CallNamePart::Argument(_) => format.push_str("<>"),
            }
        }
        format
    }
}

impl StartsHere for Call {
    /// Check that call may start at current lexer position
    fn starts_here(lexer: &mut impl Lexer) -> bool {
        lexer.try_match(Token::Id).is_ok()
    }
}

impl Ranged for Call {
    /// Get range of function call
    fn range(&self) -> std::ops::Range<usize> {
        self.name_parts.first().unwrap().range().start..self.name_parts.last().unwrap().range().end
    }
}

impl Parse for Call {
    type Err = ParseError;

    /// Parse function call using lexer
    fn parse(lexer: &mut impl Lexer) -> Result<Self, Self::Err> {
        let mut name_parts = Vec::new();

        loop {
            name_parts.push(CallNamePart::parse(lexer)?);

            let token = lexer.peek();
            if token.is_none() {
                break;
            }

            let token = token.unwrap();
            if token.is_operator() || token == Token::Newline {
                break;
            }
        }

        debug_assert!(name_parts.len() > 0);

        Ok(Call { name_parts })
    }
}
