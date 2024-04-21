use std::fmt::Display;

use derive_more::From;
use derive_visitor::DriveMut;

use crate::{
    hir::{ClassOrTrait, Function, Variable},
    syntax::{Identifier, Keyword, Ranged},
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
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct Use {
    /// Keyword `use`
    #[drive(skip)]
    pub keyword: Keyword<"use">,
    /// Path to item
    #[drive(skip)]
    pub path: Vec<Identifier>,
    /// Item, imported by use statement
    #[drive(skip)]
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

impl Ranged for Use {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.path
            .last()
            .map_or_else(|| self.keyword.end(), |p| p.end())
    }
}
