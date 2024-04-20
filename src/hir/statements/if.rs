use std::fmt::Display;

use derive_visitor::DriveMut;

use crate::syntax::{Keyword, Ranged};

use super::{Expression, Statement};

/// HIR for else-if statement
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct ElseIf {
    /// Keyword `else`
    #[drive(skip)]
    pub else_keyword: Keyword<"else">,
    /// Keyword `if`
    #[drive(skip)]
    pub if_keyword: Keyword<"if">,
    /// Condition of else-if statement
    pub condition: Expression,
    /// Body of else-if statement
    pub body: Vec<Statement>,
}

impl Ranged for ElseIf {
    fn start(&self) -> usize {
        self.else_keyword.start()
    }

    fn end(&self) -> usize {
        self.body.last().map_or(self.condition.end(), |s| s.end())
    }
}

/// HIR for else block
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct Else {
    /// Keyword `else`
    #[drive(skip)]
    pub keyword: Keyword<"else">,
    /// Body of else block
    pub body: Vec<Statement>,
}

impl Ranged for Else {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.body.last().map_or(self.keyword.end(), |s| s.end())
    }
}

/// HIR for if-statement
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct If {
    /// Keyword `if`
    #[drive(skip)]
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

impl Ranged for If {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.else_block
            .as_ref()
            .map(|else_block| else_block.end())
            .or_else(|| self.else_ifs.last().map(|else_if| else_if.end()))
            .or_else(|| self.body.last().map(|s| s.end()))
            .unwrap_or_else(|| self.condition.end())
    }
}
