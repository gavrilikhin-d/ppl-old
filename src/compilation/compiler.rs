use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    ast,
    hir::Module,
    semantics::{ModuleContext, ToHIR},
    SourceFile,
};
use log::{debug, trace};
use miette::miette;

/// Struct that compiles and caches modules
pub struct Compiler {
    /// Cache of compiled modules
    pub modules: BTreeMap<String, Arc<Module>>,
    /// Root directory of the compiler
    pub root: PathBuf,
}

impl Compiler {
    /// Create new compiler with empty cache
    pub fn new() -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime"));

        let mut compiler = Compiler::without_builtin().at(path);

        compiler.get_module("ppl").unwrap();

        compiler.at("")
    }

    /// Create new compiler without builtin module.
    /// The first module to be added will be interpreted as builtin
    pub fn without_builtin() -> Self {
        Self {
            modules: BTreeMap::new(),
            root: "".into(),
        }
    }

    /// Get builtin module, if present.
    ///
    /// Builtin module is the first module compiled
    pub fn builtin_module(&self) -> Option<&Module> {
        self.modules.first_key_value().map(|(_, m)| m.as_ref())
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
    /// let m1 = compiler.get_module("main").unwrap();
    /// let m2 = compiler.get_module("main").unwrap();
    /// assert_eq!(m1, m2);
    /// ```
    pub fn get_module(&mut self, name: &str) -> miette::Result<Arc<Module>> {
        if let Some(module) = self.modules.get(name) {
            return Ok(module.clone());
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

        let module = Module::new(SourceFile::with_path(&path).unwrap());

        let content = fs::read_to_string(&path).map_err(|e| miette!("{path:?}: {e}"))?;

        trace!(target: "steps", "Lowering to hir `{}`", path.display());
        let mut context = ModuleContext {
            module,
            compiler: self,
        };
        let hir = ast.to_hir(&mut context).map_err(|e| {
            miette::Report::from(e)
                .with_source_code(miette::NamedSource::new(path.to_string_lossy(), content))
        })?;
        debug!(target: &format!("{name}-hir"), "\n{:#}", hir);

        let module = Arc::new(hir);
        self.modules.insert(name.to_string(), module.clone());
        Ok(module)
    }
}
