use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use derive_visitor::DriveMut;
use indexmap::IndexMap;

use crate::{
    ast,
    hir::{ClassData, FunctionData, ModuleData, TraitData},
    semantics::{InsertDestructors, ModuleContext, TemporariesInserter, ToHIR},
    SourceFile,
};
use log::{debug, trace};
use miette::{bail, miette};

use super::{Package, PackageData};

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
    /// All packages across compilation process
    pub packages: IndexMap<String, PackageData>,
    /// Stack of packages being compiled
    pub package_stack: Vec<Package>,
    /// Stack of modules being compiled
    pub modules_stack: Vec<Module>,
    /// Cache of compiled modules
    pub modules: IndexMap<PathBuf, ModuleData>,
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
    /// Location of PPL package
    pub const PPL_PACKAGE: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/ppl");

    /// Create new compiler with empty cache
    pub fn new() -> Self {
        let path = Path::new(Self::PPL_PACKAGE);
        let mut compiler = Compiler::without_builtin().at(path);

        compiler.compile_package("ppl").unwrap();

        compiler.import_builtin = true;

        compiler.at("")
    }

    /// Create new compiler without builtin module.
    /// The first module to be added will be interpreted as builtin
    pub fn without_builtin() -> Self {
        Self {
            asts: Default::default(),
            packages: Default::default(),
            package_stack: Default::default(),
            modules_stack: Default::default(),
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

    /// Get current package
    pub fn current_package(&self) -> Package {
        self.package_stack
            .last()
            .cloned()
            .unwrap_or(Package::with_index(0))
    }

    /// Get current module
    pub fn current_module(&self) -> Module {
        self.modules_stack
            .last()
            .cloned()
            .unwrap_or(Module::with_index(0))
    }

    /// Get current source file
    pub fn current_file(&self) -> &SourceFile {
        self.current_module().data(self).source_file()
    }

    /// Locate module by name
    ///
    /// # Module search order
    /// 1. `{root}/src/{name}.ppl`
    /// 2. `{root}/src/{name}/mod.ppl`
    pub fn locate(&mut self, name: &str) -> miette::Result<PathBuf> {
        let variants = vec![
            self.root.join("src").join(format!("{name}.ppl")),
            self.root.join("src").join(name).join("mod.ppl"),
        ];

        variants
            .iter()
            .find(|p| p.exists())
            .cloned()
            .ok_or_else(|| miette!("Module `{name}` not found. Tried {:#?}", variants))
    }

    /// Parse module from file
    fn parse(&mut self, path: &Path) -> miette::Result<ast::Module> {
        let canonic_path = std::fs::canonicalize(path).unwrap();

        if let Some(ast) = self.asts.get(&canonic_path) {
            return Ok(ast.clone());
        }

        trace!(target: "steps", "Parsing `{}`", path.display());
        let ast = ast::Module::from_file(path)?;
        self.asts.insert(canonic_path, ast.clone());
        Ok(ast)
    }

    /// Get compiled module from cache or compile it
    ///
    /// # Module search order
    /// 1. `{root}/src/{name}.ppl`
    /// 2. `{root}/src/{name}/mod.ppl`
    pub(crate) fn compile(&mut self, name: &str) -> miette::Result<Module> {
        let path = self.locate(name)?;
        let canonic_path = std::fs::canonicalize(&path).unwrap();

        if let Some(index) = self.modules.get_index_of(&canonic_path) {
            return Ok(Module::with_index(index));
        }

        let ast = self.parse(&path)?;

        let index = self.modules.len();
        let module = Module::with_index(index);

        self.modules_stack.push(module);

        let current_package = self.current_package();
        current_package.data_mut(self).modules.push(module);

        let source_file = SourceFile::with_path(&path).unwrap();
        let data = ModuleData::new(source_file.clone());
        self.modules.insert(canonic_path, data.clone());

        trace!(target: "steps", "Lowering to hir `{}`", path.display());
        let mut context = ModuleContext::new(ModuleData::new(source_file.clone()), self);
        let mut hir = ast
            .to_hir(&mut context)
            .map_err(|e| miette::Report::from(e).with_source_code(source_file))?;
        debug!(target: &format!("{name}-hir"), "\n{:#}", hir);

        trace!(target: "steps", "Inserting destructors `{}`", path.display());
        hir.drive_mut(&mut TemporariesInserter::new(&mut context));
        hir.insert_destructors(&mut context);
        debug!(target: &format!("{name}-hir-with-destructors"), "\n{:#}", hir);

        self.modules[module.index()] = hir;

        self.modules_stack.pop();

        Ok(module)
    }

    /// Locates package by name. Returns relative path (except for `ppl` package)
    fn locate_package(&mut self, package: &str) -> miette::Result<PathBuf> {
        if package == "ppl" {
            return Ok(Self::PPL_PACKAGE.into());
        }

        let cwd = current_dir().unwrap();
        if cwd.is_dir() && cwd.ends_with(package) {
            return Ok("".into());
        }

        let dep = Path::new("dependencies").join(package);
        if dep.is_dir() {
            return Ok(dep);
        }

        bail!(
            "Package `{package}` not found in {} or {}",
            cwd.display(),
            dep.display()
        )
    }

    /// Get compiled package from cache or compile it
    pub fn compile_package(&mut self, package: &str) -> miette::Result<Package> {
        if let Some(index) = self.packages.get_index_of(package) {
            return Ok(Package::with_index(index));
        }

        let name = package.to_string();
        let index = self.packages.len();
        let package = Package::with_index(index);
        let old_root = self.root.clone();
        let root = self.locate_package(&name)?;
        self.root = root.clone();
        self.packages.insert(
            name.clone(),
            PackageData {
                root,
                name: name.clone(),
                modules: Default::default(),
                dependencies: Default::default(),
            },
        );

        self.package_stack.push(package);
        let main = self.root.join("src/main.ppl");
        let lib = self.root.join("src/lib.ppl");
        if main.exists() {
            self.compile("main")?;
        } else if lib.exists() {
            self.compile("lib")?;
        } else {
            bail!(
                "No {main} or {lib} found in package `{name}`",
                main = main.display(),
                lib = lib.display()
            );
        }
        self.package_stack.pop();
        self.root = old_root;

        Ok(package)
    }
}
