extern crate ast_derive;

use ast_derive::AST;

use derive_more::{From, TryInto};

use super::{parse_binary_expression, Expression};

use crate::syntax::{
    error::ParseError, Context, Identifier, Lexer, Parse, Ranged, StartsHere, StringWithOffset,
    Token,
};

/// Cell of function
#[derive(Debug, PartialEq, Eq, AST, Clone, From, TryInto)]
pub enum CallNamePart {
    Text(Identifier),
    Argument(Expression),
}

impl From<StringWithOffset> for CallNamePart {
    fn from(string: StringWithOffset) -> Self {
        CallNamePart::Text(string.into())
    }
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
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        if context
            .lexer
            .consume_one_of(&[Token::LBracket, Token::RBracket])
            .is_ok()
        {
            return Ok(context.lexer.string_with_offset().into());
        }

        let expr = parse_binary_expression(context)?;
        Ok(match expr {
            Expression::VariableReference(var) => var.name.into(),
            Expression::TypeReference(ty) if ty.generic_parameters.len() == 0 => ty.name.into(),
            _ => expr.into(),
        })
    }
}

/// Kind of function to call
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FnKind {
    /// Function
    Function,
    /// Operator
    Operator,
}

/// AST for function call
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Call {
    /// Kind of function to call
    pub kind: FnKind,
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
    fn starts_here(context: &mut Context<impl Lexer>) -> bool {
        context
            .lexer
            .try_match_one_of(&[Token::Id, Token::EscapedId])
            .is_ok()
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
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let mut name_parts = Vec::new();

        loop {
            name_parts.push(CallNamePart::parse(context)?);

            let token = context.lexer.peek();
            if token.map_or(true, |t| t.ends_expression()) {
                break;
            }
        }

        debug_assert!(name_parts.len() > 0);

        Ok(Call {
            kind: FnKind::Function,
            name_parts,
        })
    }
}
