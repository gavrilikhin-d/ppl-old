use std::{fmt::Display, sync::Arc};

use crate::{
    compilation::Compiler,
    hir::{Class, Function, ModuleData, TraitDeclaration, Variable},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration},
};

use super::Context;

/// Context for lowering content of module
pub struct ModuleContext<'c> {
    /// Module, which is being lowered
    pub module: ModuleData,
    /// Compiler for modules
    pub compiler: &'c mut Compiler,
}

impl Display for ModuleContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ModuleContext:")?;
        writeln!(f, "\tfor module: {}", self.module.name())
    }
}

impl<'c> ModuleContext<'c> {
    pub fn new(compiler: &'c mut Compiler) -> Self {
        Self {
            module: ModuleData::default(),
            compiler,
        }
    }
}

impl AsRef<ModuleData> for ModuleContext<'_> {
    fn as_ref(&self) -> &ModuleData {
        &self.module
    }
}

impl FindDeclaration for ModuleContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        self.compiler.builtin_module().map(|m| m as _)
    }
}

impl AddDeclaration for ModuleContext<'_> {
    fn add_type(&mut self, ty: Class) {
        self.module.add_type(ty)
    }

    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.module.add_trait(tr)
    }

    fn add_function(&mut self, f: Function) {
        self.module.add_function(f)
    }

    fn add_variable(&mut self, v: Variable) {
        self.module.add_variable(v)
    }
}

impl Context for ModuleContext<'_> {
    fn compiler(&self) -> &Compiler {
        self.compiler
    }

    fn compiler_mut(&mut self) -> &mut Compiler {
        self.compiler
    }

    fn module(&self) -> &ModuleData {
        &self.module
    }

    fn module_mut(&mut self) -> &mut ModuleData {
        &mut self.module
    }
}
