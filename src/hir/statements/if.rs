use std::fmt::Display;

use super::{Expression, Statement};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElseIf {
    /// Condition of else-if statement
    pub condition: Expression,
    /// Body of else-if statement
    pub body: Vec<Statement>,
}

/// AST for if-statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct If {
    /// Condition of if-statement
    pub condition: Expression,
    /// Body of if-statement
    pub body: Vec<Statement>,
    /// Else-if statements
    pub else_ifs: Vec<ElseIf>,
    /// Else block
    pub else_block: Vec<Statement>,
}

impl Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "if {}:", self.condition)?;
        for statement in &self.body {
            writeln!(f, "\t{}", statement)?;
        }
        for else_if in &self.else_ifs {
            writeln!(f, "else if {}:", else_if.condition)?;
            for statement in &else_if.body {
                writeln!(f, "\t{}", statement)?;
            }
        }
        if !self.else_block.is_empty() {
            writeln!(f, "else:")?;
            for statement in &self.else_block {
                writeln!(f, "\t{}", statement)?;
            }
        }
        Ok(())
    }
}
