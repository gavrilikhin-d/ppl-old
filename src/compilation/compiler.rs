use std::{
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;

use crate::{
    ast,
    hir::ModuleData,
    named::Named,
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

/// Struct that compiles and caches modules
pub struct Compiler {
    /// Cache of compiled modules
    pub modules: IndexMap<String, ModuleData>,
    /// Root directory of the compiler
    pub root: PathBuf,
}

impl Compiler {
    /// Create new compiler with empty cache
    pub fn new() -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime"));

        let mut compiler = Compiler::without_builtin().at(path);

        compiler.compile("ppl").unwrap();

        compiler.at("")
    }

    /// Create new compiler without builtin module.
    /// The first module to be added will be interpreted as builtin
    pub fn without_builtin() -> Self {
        Self {
            modules: IndexMap::new(),
            root: "".into(),
        }
    }

    /// Get builtin module, if present.
    ///
    /// Builtin module is the first module compiled
    pub fn builtin_module(&self) -> Option<&ModuleData> {
        self.modules.values().next().map(|m| {
            debug_assert!(m.name() == "ppl", "Wrong module used as builtin");
            m
        })
    }

    /// Return compiler with root directory set to `root`
    pub fn at(self, root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..self
        }
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
    /// let m2 = compiler.compule("main").unwrap();
    /// assert_eq!(m1, m2);
    /// ```
    pub fn compile<'c>(&'c mut self, name: &str) -> miette::Result<Module> {
        if let Some(index) = self.modules.get_index_of(name) {
            return Ok(Module::with_index(index));
        }

        let variants = vec![
            self.root.join(format!("{name}.ppl")),
            self.root.join(name).join("mod.ppl"),
        ];

        let path = variants
            .iter()
            .filter(|p| p.exists())
            .next()
            .cloned()
            .ok_or_else(|| miette!("Module `{name}` not found. Tried {:#?}", variants))?;

        trace!(target: "steps", "Parsing `{}`", path.display());
        let ast = ast::Module::from_file(&path)?;
        debug!(target: &format!("{name}-ast"), "\n{:#?}", ast);

        let module = ModuleData::new(SourceFile::with_path(&path).unwrap());

        let content = fs::read_to_string(&path).map_err(|e| miette!("{path:?}: {e}"))?;

        trace!(target: "steps", "Lowering to hir `{}`", path.display());
        let mut context = ModuleContext {
            module,
            compiler: self,
        };
        let mut hir = ast.to_hir(&mut context).map_err(|e| {
            miette::Report::from(e)
                .with_source_code(miette::NamedSource::new(path.to_string_lossy(), content))
        })?;
        debug!(target: &format!("{name}-hir"), "\n{:#}", hir);

        trace!(target: "steps", "Inserting destructors `{}`", path.display());
        hir.insert_destructors(&mut context);
        debug!(target: &format!("{name}-hir-with-destructors"), "\n{:#}", hir);

        let index = self.modules.len();
        self.modules.insert(name.to_string(), hir);
        Ok(Module::with_index(index))
    }
}
