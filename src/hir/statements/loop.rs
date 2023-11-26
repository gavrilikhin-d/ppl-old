use std::fmt::Display;

use crate::hir::Statement;

/// Infinite loop
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Loop {
    /// Body of a loop
    pub body: Vec<Statement>,
}

impl Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "loop:")?;
        for statement in &self.body {
            writeln!(f, "\t{}", statement)?;
        }
        Ok(())
    }
}
