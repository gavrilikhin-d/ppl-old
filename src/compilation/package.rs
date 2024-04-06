use std::path::PathBuf;

use indexmap::IndexMap;

use super::{Compiler, Module};

/// Package index inside a Compiler
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Package {
    index: usize,
}

impl Package {
    /// Get package with specified index
    pub fn with_index(index: usize) -> Self {
        Self { index }
    }

    /// Convert to underlying index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Access data of a package
    pub fn data<'c>(&self, compiler: &'c Compiler) -> &'c PackageData {
        compiler.packages.get_index(self.index()).unwrap().1
    }
}

/// Package data structure
pub struct PackageData {
    /// Name of the package
    pub name: String,
    /// List of modules in the package
    pub modules: IndexMap<PathBuf, Module>,
}
