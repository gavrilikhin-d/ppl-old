use std::fmt::Display;

use derive_visitor::DriveMut;

use crate::{
    hir::{Expression, Statement},
    syntax::{Keyword, Ranged},
};

/// While loop
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct While {
    /// Keyword `while`
    #[drive(skip)]
    pub keyword: Keyword<"while">,
    /// Condition of a loop
    pub condition: Expression,
    /// Body of a loop
    pub body: Vec<Statement>,
}

impl Display for While {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = f.width().unwrap_or(0);
        let new_indent = indent + 1;

        let indent = "\t".repeat(indent);
        write!(f, "{indent}")?;

        writeln!(f, "while {}:", self.condition)?;
        for statement in &self.body {
            writeln!(f, "{statement:#new_indent$}")?;
        }
        Ok(())
    }
}

impl Ranged for While {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.body.last().map_or(self.condition.end(), |s| s.end())
    }
}
