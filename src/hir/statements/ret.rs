use std::fmt::Display;

use crate::{hir::Expression, syntax::{Keyword, Ranged}};

/// Return statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Return {
    /// Implicit return
    Implicit {
        /// Returned value
        value: Expression,
    },
    /// Explicit return
    Explicit {
        /// Keyword `return`
        keyword: Keyword<"return">,
        /// Returned value
        value: Option<Expression>,
    },
}

impl Return {
    /// Keyword `return`
    pub fn keyword(&self) -> Option<Keyword<"return">> {
        match self {
            Return::Implicit { .. } => None,
            Return::Explicit { keyword, .. } => Some(*keyword),
        }
    }

    /// Returned value
    pub fn value(&self) -> Option<&Expression> {
        match self {
            Return::Implicit { value } => Some(value),
            Return::Explicit { value, .. } => value.as_ref(),
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut Expression> {
        match self {
            Return::Implicit { value } => Some(value),
            Return::Explicit { value, .. } => value.as_mut(),
        }
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        if let Some(value) = self.value() {
            write!(f, "return {}", value)
        } else {
            write!(f, "return")
        }
    }
}

impl Ranged for Return {
    fn start(&self) -> usize {
        match self {
            Return::Implicit { value } => value.start(),
            Return::Explicit { keyword, .. } => keyword.start(),
        }
    }

    fn end(&self) -> usize {
        match self {
            Return::Implicit { value } => value.end(),
            Return::Explicit { keyword, value,  } => value.as_ref().map_or(keyword.end(), |v| v.end()),
        }
    }
}
