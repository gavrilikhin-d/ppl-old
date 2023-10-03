use std::{fs, path::Path};

use miette::miette;

use super::Statement;

extern crate ast_derive;
use ast_derive::AST;

use crate::syntax::{error::ParseError, Context, Lexer, Parse};

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, AST, Clone)]
pub struct Module {
    /// Statements in module
    pub statements: Vec<Statement>,
}

impl Parse for Module {
    type Err = ParseError;

    /// Parse all statements in module
    fn parse(context: &mut Context<impl Lexer>) -> Result<Self, Self::Err> {
        let mut statements = Vec::new();

        context.lexer.skip_spaces();
        while context.lexer.peek().is_some() {
            statements.push(Statement::parse(context)?);
            context.lexer.skip_spaces();
        }

        Ok(Module { statements })
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
