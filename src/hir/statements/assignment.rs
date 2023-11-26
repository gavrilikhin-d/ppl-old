use std::fmt::Display;

use crate::hir::Expression;

/// Assignment of a value to a reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
    /// Reference to assign to
    pub target: Expression,
    /// Value to assign
    pub value: Expression,
}

impl Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}
