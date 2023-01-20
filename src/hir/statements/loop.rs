use crate::hir::Statement;

/// Infinite loop
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Loop {
    /// Body of a loop
    pub body: Vec<Statement>
}
