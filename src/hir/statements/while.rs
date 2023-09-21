use crate::hir::{Expression, Statement};

/// While loop
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct While {
    /// Condition of a loop
    pub condition: Expression,
    /// Body of a loop
    pub body: Vec<Statement>,
}
