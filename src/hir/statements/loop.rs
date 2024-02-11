use std::fmt::Display;

use crate::{hir::Statement, syntax::{Keyword, Ranged}};

/// Infinite loop
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Loop {
    /// Keyword `loop`
    pub keyword: Keyword<"loop">,
    /// Body of a loop
    pub body: Vec<Statement>,
}

impl Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = f.width().unwrap_or(0);
        let new_indent = indent + 1;

        let indent = "\t".repeat(indent);
        write!(f, "{indent}")?;

        writeln!(f, "loop:")?;
        for statement in &self.body {
            writeln!(f, "{statement:#new_indent$}")?;
        }
        Ok(())
    }
}

impl Ranged for Loop {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.body.last().map_or(self.keyword.end(), |s| s.end())
    }
}
