use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::hir::Module;
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

        let path = vec![
            self.root.join(format!("{name}.ppl")),
            self.root.join(name).join("mod.ppl"),
        ]
        .iter()
        .filter(|p| p.exists())
        .next()
        .cloned()
        .ok_or_else(|| miette!("Module {name} not found"))?;

        let module = Arc::new(Module::from_file_with_builtin(&path, self.is_builtin)?);
        self.modules.insert(name.to_string(), module.clone());
        Ok(module)
    }
}
