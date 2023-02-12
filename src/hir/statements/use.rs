use std::sync::Arc;

use derive_more::From;

use crate::{
    hir::{ClassOrTrait, Function, VariableDeclaration},
    syntax::StringWithOffset,
};

/// Item, imported by use statement
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ImportedItem {
    ClassOrTrait(ClassOrTrait),
    Function(Function),
    Variable(Arc<VariableDeclaration>),
}

/// Use statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Use {
    /// Path to item
    pub path: Vec<StringWithOffset>,
    /// Item, imported by use statement
    pub imported_item: ImportedItem,
}
