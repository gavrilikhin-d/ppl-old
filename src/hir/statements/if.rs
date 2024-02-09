use std::fmt::Display;

use crate::syntax::Keyword;

use super::{Expression, Statement};

/// HIR for else-if statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ElseIf {
    /// Keyword `else`
    pub else_keyword: Keyword<"else">,
    /// Keyword `if`
    pub if_keyword: Keyword<"if">,
    /// Condition of else-if statement
    pub condition: Expression,
    /// Body of else-if statement
    pub body: Vec<Statement>,
}

/// HIR for else block
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Else {
    /// Keyword `else`
    pub keyword: Keyword<"else">,
    /// Body of else block
    pub body: Vec<Statement>,
}

/// HIR for if-statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct If {
    /// Keyword `if`
    pub keyword: Keyword<"if">,
    /// Condition of if-statement
    pub condition: Expression,
    /// Body of if-statement
    pub body: Vec<Statement>,
    /// Else-if statements
    pub else_ifs: Vec<ElseIf>,
    /// Else block
    pub else_block: Option<Else>,
}

impl Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = f.width().unwrap_or(0);
        let new_indent = indent + 1;

        let indent = "\t".repeat(indent);

        write!(f, "{indent}")?;
        writeln!(f, "if {}:", self.condition)?;
        for statement in &self.body {
            writeln!(f, "{statement:#new_indent$}")?;
        }
        for else_if in &self.else_ifs {
            write!(f, "{indent}")?;
            writeln!(f, "else if {}:", else_if.condition)?;
            for statement in &else_if.body {
                writeln!(f, "{statement:#new_indent$}")?;
            }
        }
        if let Some(else_block) = &self.else_block {
            write!(f, "{indent}")?;
            writeln!(f, "else:")?;
            for statement in &else_block.body {
                writeln!(f, "{statement:#new_indent$}")?;
            }
        }
        Ok(())
    }
}
