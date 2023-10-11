use std::{borrow::Cow, fmt::Display};

use crate::{named::Named, syntax::StringWithOffset};

use super::Function;

/// Declaration of a trait
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TraitDeclaration {
    /// Trait's name
    pub name: StringWithOffset,
    /// Associated functions
    pub functions: Vec<Function>,
}

impl TraitDeclaration {
    /// Iterate over all functions with `n` name parts
    pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = &Function> + '_ {
        self.functions
            .iter()
            .filter(move |f| f.name_parts().len() == n)
    }
}

impl Named for TraitDeclaration {
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for TraitDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
