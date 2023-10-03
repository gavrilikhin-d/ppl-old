use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use crate::{
    ast,
    hir::Module,
    semantics::{ASTLoweringWithinModule, ModuleContext},
};
use log::debug;
use miette::miette;

/// Struct that compiles and caches modules
pub struct Compiler {
    /// Cache of compiled modules
    pub modules: HashMap<String, Arc<Module>>,
    /// Is this a compiler for builtin modules?
    pub is_builtin: bool,
    /// Root directory of the compiler
    pub root: PathBuf,
}

impl Compiler {
    /// Create new compiler with empty cache
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            is_builtin: false,
            root: "".into(),
        }
    }

    /// Create new compiler for builtin modules
    pub fn for_builtin() -> Self {
        Self {
            modules: HashMap::new(),
            is_builtin: true,
            root: "".into(),
        }
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

        let ast = ast::Module::from_file(&path)?;
        debug!(target: "ast", "{:#?}", ast);

        let mut module = Module::new(
            path.file_stem().unwrap().to_str().unwrap(),
            path.to_str().unwrap(),
        );
        module.is_builtin = self.is_builtin;

        let content = fs::read_to_string(&path).map_err(|e| miette!("{path:?}: {e}"))?;

        let mut context = ModuleContext {
            module,
            compiler: self,
        };
        ast.lower_to_hir_within_context(&mut context).map_err(|e| {
            miette::Report::from(e)
                .with_source_code(miette::NamedSource::new(path.to_string_lossy(), content))
        })?;
        debug!(target: "hir", "{:#?}", context.module);

        let module = Arc::new(context.module);
        self.modules.insert(name.to_string(), module.clone());
        Ok(module)
    }
}
