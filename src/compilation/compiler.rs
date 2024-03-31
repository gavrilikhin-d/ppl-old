use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::{
    ast,
    hir::{ClassData, FunctionData, ModuleData, TraitData},
    semantics::{InsertDestructors, ModuleContext, ToHIR},
    SourceFile,
};
use log::{debug, trace};
use miette::miette;

/// Module index inside a Compiler
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Module {
    index: usize,
}

impl Module {
    /// Get module with specified
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    /// Convert to underlying index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Access data of a bodule
    pub fn data<'c>(&self, compiler: &'c Compiler) -> &'c ModuleData {
        compiler.modules.get_index(self.index()).unwrap().1
    }
}

/// Index of a function in Compiler
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Function {
    index: usize,
}

impl Function {
    /// Point at function with specified index
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    /// Get underlying index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Access data of a function
    pub fn data<'c>(&self, compiler: &'c Compiler) -> &'c FunctionData {
        &compiler.functions[self.index()]
    }

    /// Access data of a function for mutation
    pub fn data_mut<'c>(&self, compiler: &'c mut Compiler) -> &'c mut FunctionData {
        &mut compiler.functions[self.index()]
    }
}

/// Struct that compiles and caches modules
pub struct Compiler {
    /// ASTs of all modules
    pub asts: IndexMap<PathBuf, ast::Module>,
    /// Cache of compiled modules
    pub modules: IndexMap<String, ModuleData>,
    /// Functions from all modules
    pub functions: Vec<FunctionData>,
    /// Classes from all modules
    pub classes: IndexMap<String, ClassData>,
    /// Traits from all modules
    pub traits: IndexMap<String, TraitData>,
    /// Root directory of the compiler
    pub root: PathBuf,
    /// Import builtin module
    pub import_builtin: bool,
}

impl Compiler {
    /// Create new compiler with empty cache
    pub fn new() -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime"));

        let mut compiler = Compiler::without_builtin().at(path);

        compiler.compile("ppl").unwrap();

        compiler.import_builtin = true;

        compiler.at("")
    }

    /// Create new compiler without builtin module.
    /// The first module to be added will be interpreted as builtin
    pub fn without_builtin() -> Self {
        Self {
            asts: Default::default(),
            modules: Default::default(),
            functions: Default::default(),
            classes: Default::default(),
            traits: Default::default(),
            root: Default::default(),
            import_builtin: false,
        }
    }

    /// Return compiler with root directory set to `root`
    pub fn at(self, root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..self
        }
    }

    /// Locate module by name
    ///
    /// # Module search order
    /// 1. `{root}/{name}.ppl`
    /// 2. `{root}/{name}/mod.ppl`
    pub fn locate(&mut self, name: &str) -> miette::Result<PathBuf> {
        let variants = [
            self.root.join(format!("{name}.ppl")),
            self.root.join(name).join("mod.ppl"),
        ];

        variants
            .iter()
            .find(|p| p.exists())
            .cloned()
            .ok_or_else(|| miette!("Module `{name}` not found. Tried {:#?}", variants))
    }

    /// Parse module from file
    fn parse(&mut self, path: &Path) -> miette::Result<ast::Module> {
        if let Some(ast) = self.asts.get(path) {
            return Ok(ast.clone());
        }

        trace!(target: "steps", "Parsing `{}`", path.display());
        let ast = ast::Module::from_file(path)?;
        self.asts.insert(path.to_path_buf(), ast.clone());
        Ok(ast)
    }

    /// Get compiled module from cache or compile it
    ///
    /// # Module search order
    /// 1. `{root}/{name}.ppl`
    /// 2. `{root}/{name}/mod.ppl`
    ///
    /// # Example
    /// ```no_run
    /// use ppl::compilation::Compiler;
    ///
    /// let mut compiler = Compiler::new().at("src");
    /// let m1 = compiler.compile("main").unwrap();
    /// let m2 = compiler.compile("main").unwrap();
    /// assert_eq!(m1, m2);
    /// ```
    pub fn compile(&mut self, name: &str) -> miette::Result<Module> {
        if let Some(index) = self.modules.get_index_of(name) {
            return Ok(Module::with_index(index));
        }

        let path = self.locate(name)?;

        let ast = self.parse(&path)?;

        let source_file = SourceFile::with_path(&path).unwrap();

        trace!(target: "steps", "Lowering to hir `{}`", path.display());
        let mut context = ModuleContext::new(ModuleData::new(source_file.clone()), self);
        let mut hir = ast
            .to_hir(&mut context)
            .map_err(|e| miette::Report::from(e).with_source_code(source_file))?;
        debug!(target: &format!("{name}-hir"), "\n{:#}", hir);

        trace!(target: "steps", "Inserting destructors `{}`", path.display());
        hir.insert_destructors(&mut context);
        debug!(target: &format!("{name}-hir-with-destructors"), "\n{:#}", hir);

        let index = self.modules.len();
        self.modules.insert(name.to_string(), hir);
        Ok(Module::with_index(index))
    }
}
