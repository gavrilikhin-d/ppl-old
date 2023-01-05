use crate::hir::Expression;

/// Assignment of a value to a
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
    /// Variable to assign to
    pub target: Expression,
    /// Value to assign
    pub value: Expression,
}
