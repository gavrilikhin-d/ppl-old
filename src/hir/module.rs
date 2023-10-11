use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use derive_more::From;

use crate::compilation::Compiler;
use crate::hir::{Statement, TypeDeclaration, VariableDeclaration};
use crate::named::Named;
use std::sync::{Arc, LazyLock};

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
    /// Name of the module
    pub name: String,

    /// Filename of module
    pub filename: String,

    /// Is this a builtin module?
    pub is_builtin: bool,

    /// Variables, visible in this module
    pub variables: BTreeMap<Name, Arc<VariableDeclaration>>,

    /// Types, visible in this module
    pub types: BTreeMap<Name, ClassOrTrait>,

    /// Functions, visible in this module
    pub functions: BTreeMap<Format, BTreeMap<Name, Function>>,

    /// Statements in this module
    pub statements: Vec<Statement>,
}

static BUILTIN: LazyLock<Module> = LazyLock::new(|| Module::create_builtin());

impl Module {
    /// Create an empty module
    pub fn new(name: &str, filename: &str) -> Self {
        Self {
            name: name.to_string(),
            filename: filename.to_string(),
            is_builtin: false,
            variables: BTreeMap::new(),
            types: BTreeMap::new(),
            functions: BTreeMap::new(),
            statements: vec![],
        }
    }

    /// Create builtin module
    pub(crate) fn create_builtin() -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime"));

        let module = Compiler::for_builtin().at(path).get_module("ppl").unwrap();

        Arc::into_inner(module).unwrap()
    }

    /// Get builtin module
    ///
    /// # Example
    /// ```
    /// use ppl::hir::Module;
    ///
    /// let module = Module::builtin();
    /// ```
    pub fn builtin() -> &'static Self {
        &BUILTIN
    }

    /// Insert function to module
    pub fn insert_function(&mut self, function: Function) {
        let set = self
            .functions
            .entry(function.name_format().to_string())
            .or_insert_with(BTreeMap::new);
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
        self.name.as_str().into()
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new("main", "")
    }
}
