use std::collections::HashMap;
use std::path::Path;

use derive_more::From;

use crate::ast;
use crate::hir::{Statement, TypeDeclaration, VariableDeclaration};
use crate::named::Named;
use crate::semantics::{ASTLoweringWithinModule, ModuleContext};
use miette::miette;
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
    fn name(&self) -> &str {
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
    pub variables: HashMap<Name, Arc<VariableDeclaration>>,

    /// Types, visible in this module
    pub types: HashMap<Name, ClassOrTrait>,

    /// Functions, visible in this module
    pub functions: HashMap<Format, HashMap<Name, Function>>,

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
            variables: HashMap::new(),
            types: HashMap::new(),
            functions: HashMap::new(),
            statements: vec![],
        }
    }

    /// Create module from file with providing builtin module
    fn from_file_with_builtin(path: &Path, is_builtin: bool) -> miette::Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| miette!("{path:?}: {e}"))?;

        let ast = content.parse::<ast::Module>()?;

        let mut module = Module::new(
            path.file_stem().unwrap().to_str().unwrap(),
            path.to_str().unwrap(),
        );
        module.is_builtin = is_builtin;

        let mut context = ModuleContext { module };
        ast.lower_to_hir_within_context(&mut context)?;

        Ok(context.module)
    }

    /// Create module from file
    pub fn from_file(path: &Path) -> miette::Result<Self> {
        Module::from_file_with_builtin(path, false)
    }

    /// Create builtin module
    fn create_builtin_from_file(path: &Path) -> miette::Result<Self> {
        Module::from_file_with_builtin(path, true)
    }

    /// Create builtin module
    pub(crate) fn create_builtin() -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/ppl.ppl"));

        let module = Self::create_builtin_from_file(path);
        if let Err(err) = module {
            panic!(
                "Error in builtin module: {:?}",
                err.with_source_code(miette::NamedSource::new(
                    path.file_name().unwrap().to_str().unwrap(),
                    std::fs::read_to_string(path).unwrap()
                ))
            );
        }

        module.unwrap()
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
            .or_insert_with(HashMap::new);
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
    fn name(&self) -> &str {
        &self.name
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new("main", "")
    }
}
