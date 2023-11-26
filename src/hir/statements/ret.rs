use std::fmt::Display;

use crate::hir::Expression;

/// Return statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Return {
    /// Returned value
    pub value: Option<Expression>,
}

impl Display for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        if let Some(value) = &self.value {
            write!(f, "return {}", value)
        } else {
            write!(f, "return")
        }
    }
}
