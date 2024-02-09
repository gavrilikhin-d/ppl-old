use std::fmt::Display;

use derive_more::From;

use crate::{
    hir::{ClassOrTrait, Function, Variable},
    syntax::{Identifier, Keyword},
};

/// Item, imported by use statement
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ImportedItem {
    ClassOrTrait(ClassOrTrait),
    Function(Function),
    Variable(Variable),
    All,
}

/// Use statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Use {
    /// Keyword `use`
    pub keyword: Keyword<"use">,
    /// Path to item
    pub path: Vec<Identifier>,
    /// Item, imported by use statement
    pub imported_item: ImportedItem,
}

impl Display for Use {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        write!(
            f,
            "use {}",
            self.path
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}
