use std::{collections::HashMap, sync::Arc};

use crate::hir::Module;

/// Struct that compiles and caches modules
pub struct Compiler {
    /// Cache of compiled modules
    pub modules: HashMap<String, Arc<Module>>,
}

impl Compiler {
    /// Create new compiler with empty cache
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
}
