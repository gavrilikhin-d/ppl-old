use derive_visitor::DriveMut;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::fmt::Display;

use derive_more::From;
use miette::NamedSource;

use crate::hir::{Statement, Variable};
use crate::named::Named;
use crate::DataHolder;
use crate::SourceFile;

use super::{Class, Function, Trait, Type};

use crate::hir::generic::Generic;

/// Class or trait
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum ClassOrTrait {
    /// Class declaration
    Class(Class),
    /// Trait declaration
    Trait(Trait),
}

impl Display for ClassOrTrait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ClassOrTrait::*;
        match self {
            Class(c) => Display::fmt(c, f),
            Trait(t) => Display::fmt(t, f),
        }
    }
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
            ClassOrTrait::Class(c) => c.read().unwrap().name().to_string().into(),
            ClassOrTrait::Trait(t) => t.read().unwrap().name().to_string().into(),
        }
    }
}

pub type Format = String;
pub type Name = String;

/// Module with PPL code
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct ModuleData {
    /// Source file for this module
    #[drive(skip)]
    pub source_file: SourceFile,

    /// Variables, visible in this module
    #[drive(skip)]
    pub variables: IndexMap<Name, Variable>,

    /// Types, visible in this module
    #[drive(skip)]
    pub types: IndexMap<Name, ClassOrTrait>,

    /// Functions, visible in this module
    #[drive(skip)]
    pub functions: IndexMap<Format, IndexMap<Name, Function>>,

    /// Monomorphized instances of functions
    #[drive(skip)]
    pub monomorphized_functions: Vec<Function>,

    /// Statements in this module
    pub statements: Vec<Statement>,
}

impl Display for ModuleData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            writeln!(f, "{:#}", statement)?;
        }
        writeln!(f, "\n==MONOMORPHIZED==\n")?;
        for fun in self
            .monomorphized_functions
            .iter()
            .filter(|f| !f.read().unwrap().is_generic())
        {
            writeln!(f, "{fun}")?;
        }

        Ok(())
    }
}

impl ModuleData {
    /// Create an empty module
    pub fn new(source_file: SourceFile) -> Self {
        Self {
            source_file,
            variables: IndexMap::new(),
            types: IndexMap::new(),
            functions: IndexMap::new(),
            monomorphized_functions: vec![],
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
            .entry(function.read().unwrap().name_format().to_string())
            .or_insert_with(IndexMap::new);
        set.insert(function.name().to_string(), function.into());
    }

    /// Define previously declared function
    pub fn define_function(&mut self, function: Function) {
        let format = function.read().unwrap().name_format().to_string();
        self.functions
            .get_mut(&format)
            .unwrap()
            .insert(function.name().to_string(), function.into());
    }

    /// Iterate all functions
    pub fn iter_functions(&self) -> impl Iterator<Item = &Function> + '_ {
        self.functions.values().flat_map(|m| m.values())
    }

    /// Iterate all functions mut
    pub fn iter_functions_mut(&mut self) -> impl Iterator<Item = &mut Function> + '_ {
        self.functions.values_mut().flat_map(|m| m.values_mut())
    }

    /// Iterate over all functions with `n` name parts
    pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = &Function> + '_ {
        self.iter_functions()
            .filter(move |f| f.read().unwrap().name_parts().len() == n)
    }
}

impl Named for ModuleData {
    fn name(&self) -> Cow<'_, str> {
        self.source_file
            .path()
            .file_stem()
            .map(|path| path.to_string_lossy())
            .unwrap_or_else(|| self.source_file.name().into())
    }
}

impl Default for ModuleData {
    fn default() -> Self {
        Self::new(SourceFile::in_memory(NamedSource::new(
            "main",
            "".to_string(),
        )))
    }
}
