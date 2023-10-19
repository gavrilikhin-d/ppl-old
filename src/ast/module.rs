use std::{fs, path::Path};

use miette::miette;

use super::Statement;

extern crate ast_derive;
use ast_derive::AST;

use crate::{
    syntax::{
        error::{ExtraToken, ParseError},
        Context, Lexer, Parse,
    },
    ErrVec,
};

impl From<ExtraToken> for ErrVec<ParseError> {
    fn from(value: ExtraToken) -> Self {
        ErrVec {
            errors: vec![value.into()],
        }
    }
}

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Module {
    /// Statements in module
    pub statements: Vec<Statement>,
}

impl Parse for Module {
    type Err = ErrVec<ParseError>;

    /// Parse all statements in module
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let mut errors = Vec::new();
        let mut statements = Vec::new();

        context.lexer.skip_spaces();
        while context.lexer.peek().is_some() {
            let res = Statement::parse(context);
            match res {
                Ok(stmt) => {
                    statements.push(stmt);
                    context.lexer.skip_spaces();
                }
                Err(e) => {
                    errors.push(e);
                    context.lexer.skip_till_next_line();
                }
            }
        }

        if errors.is_empty() {
            Ok(Module { statements })
        } else {
            Err(errors.into())
        }
    }
}

impl Module {
    /// Parse module from file
    pub fn from_file(path: &Path) -> miette::Result<Self> {
        let source = fs::read_to_string(path).map_err(|e| miette!("{path:?}: {e}"))?;
        source.parse().map_err(|e| {
            miette::Report::from(e).with_source_code(miette::NamedSource::new(
                path.to_string_lossy(),
                source.clone(),
            ))
        })
    }
}
