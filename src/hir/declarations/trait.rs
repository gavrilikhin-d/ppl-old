use std::{
    borrow::Cow,
    fmt::Display,
    hash::{Hash, Hasher},
    sync::Arc,
};

use indexmap::IndexMap;

use crate::{named::Named, syntax::Identifier, AddSourceLocation};

use super::Function;

/// Declaration of a trait
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TraitDeclaration {
    /// Trait's name
    pub name: Identifier,
    /// Associated functions
    pub functions: IndexMap<String, Arc<Function>>,
}

impl TraitDeclaration {
    /// Iterate over all functions with `n` name parts
    pub fn functions_with_n_name_parts(
        &self,
        n: usize,
    ) -> impl Iterator<Item = &Arc<Function>> + '_ {
        self.functions
            .values()
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
        if f.alternate() {
            let indent = f.width().unwrap_or(0);
            let new_indent = indent + 1;

            let indent = "\t".repeat(indent);
            write!(f, "{indent}")?;

            writeln!(f, "trait {}:", self.name())?;
            for function in self.functions.values() {
                writeln!(f, "{function:#new_indent$}")?;
            }
        } else {
            write!(f, "{}", self.name())?;
        }
        Ok(())
    }
}

impl AddSourceLocation for Arc<TraitDeclaration> {}

impl Hash for TraitDeclaration {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
