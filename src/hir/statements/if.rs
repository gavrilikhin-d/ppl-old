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
