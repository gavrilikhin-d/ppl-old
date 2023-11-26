use std::fmt::Display;

use crate::hir::{Expression, Statement};

/// While loop
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct While {
    /// Condition of a loop
    pub condition: Expression,
    /// Body of a loop
    pub body: Vec<Statement>,
}

impl Display for While {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "while {}:", self.condition)?;
        for statement in &self.body {
            writeln!(f, "\t{}", statement)?;
        }
        Ok(())
    }
}
