use crate::hir::Expression;

/// Assignment of a value to a reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
    /// Reference to assign to
    pub target: Expression,
    /// Value to assign
    pub value: Expression,
}
