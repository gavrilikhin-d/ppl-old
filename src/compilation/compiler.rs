use std::collections::HashMap;

use crate::hir::Module;

pub struct Compiler {
    pub modules: HashMap<String, Module>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }
}
