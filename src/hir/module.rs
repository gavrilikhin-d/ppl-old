use indexmap::IndexMap;
use std::borrow::Cow;
use std::fmt::Display;

use derive_more::From;
use miette::NamedSource;

use crate::hir::{Statement, TypeDeclaration, VariableDeclaration};
use crate::named::Named;
use crate::SourceFile;
use std::sync::Arc;

use super::{Function, FunctionDefinition, TraitDeclaration, Type};

/// Class or trait
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ClassOrTrait {
    /// Class declaration
    Class(Arc<TypeDeclaration>),
    /// Trait declaration
    Trait(Arc<TraitDeclaration>),
}

impl From<ClassOrTrait> for Type {
    fn from(class_or_trait: ClassOrTrait) -> Self {
        match class_or_trait {
            ClassOrTrait::Class(c) => Type::Class(c),
            ClassOrTrait::Trait(t) => Type::Trait(t),
        }
    }
}

impl Named for ClassOrTrait {
    fn name(&self) -> Cow<'_, str> {
        match self {
            ClassOrTrait::Class(c) => c.name(),
            ClassOrTrait::Trait(t) => t.name(),
        }
    }
}

pub type Format = String;
pub type Name = String;

/// Module with PPL code
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Module {
    /// Source file for this module
    pub source_file: SourceFile,

    /// Variables, visible in this module
    pub variables: IndexMap<Name, Arc<VariableDeclaration>>,

    /// Types, visible in this module
    pub types: IndexMap<Name, ClassOrTrait>,

    /// Functions, visible in this module
    pub functions: IndexMap<Format, IndexMap<Name, Function>>,

    /// Statements in this module
    pub statements: Vec<Statement>,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{:#}", statement)?;
        }
        Ok(())
    }
}

impl Module {
    /// Create an empty module
    pub fn new(source_file: SourceFile) -> Self {
        Self {
            source_file,
            variables: IndexMap::new(),
            types: IndexMap::new(),
            functions: IndexMap::new(),
            statements: vec![],
        }
    }

    /// Get source file for this module
    pub fn source_file(&self) -> &SourceFile {
        &self.source_file
    }

    /// Insert function to module
    pub fn insert_function(&mut self, function: Function) {
        let set = self
            .functions
            .entry(function.name_format().to_string())
            .or_insert_with(IndexMap::new);
        set.insert(function.name().to_string(), function.into());
    }

    /// Define previously declared function
    pub fn define_function(&mut self, function: Arc<FunctionDefinition>) {
        self.functions
            .get_mut(function.name_format())
            .unwrap()
            .insert(function.name().to_string(), function.into());
    }

    /// Iterate all functions
    pub fn iter_functions(&self) -> impl Iterator<Item = &Function> + '_ {
        self.functions.values().flat_map(|m| m.values())
    }

    /// Iterate over all functions with `n` name parts
    pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = &Function> + '_ {
        self.iter_functions()
            .filter(move |f| f.name_parts().len() == n)
    }
}

impl Named for Module {
    fn name(&self) -> Cow<'_, str> {
        self.source_file
            .path()
            .file_stem()
            .map(|path| path.to_string_lossy())
            .unwrap_or_else(|| self.source_file.name().into())
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new(SourceFile::in_memory(NamedSource::new(
            "main",
            "".to_string(),
        )))
    }
}
