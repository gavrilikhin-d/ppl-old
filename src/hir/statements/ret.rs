use crate::hir::Expression;

/// Return statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Return {
    /// Returned value
    pub value: Option<Expression>,
}
