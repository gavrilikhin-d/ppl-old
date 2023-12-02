use std::{fmt::Display, sync::Arc};

use crate::{
    compilation::Compiler,
    hir::{Basename, Function, Module, TraitDeclaration, TypeDeclaration, VariableDeclaration},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration},
};

use super::Context;

/// Context for lowering content of module
pub struct ModuleContext<'c> {
    /// Module, which is being lowered
    pub module: Module,
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
            module: Module::default(),
            compiler,
        }
    }
}

impl AsRef<Module> for ModuleContext<'_> {
    fn as_ref(&self) -> &Module {
        &self.module
    }
}

impl FindDeclaration for ModuleContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        self.compiler.builtin_module().map(|m| m as _)
    }
}

impl AddDeclaration for ModuleContext<'_> {
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        self.module
            .types
            .insert(ty.basename().to_string(), ty.into());
    }

    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.module.types.insert(tr.name().to_string(), tr.into());
    }

    fn add_function(&mut self, f: Function) {
        self.module.insert_function(f);
    }

    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        self.module.variables.insert(v.name().to_string(), v);
    }
}

impl Context for ModuleContext<'_> {
    fn compiler(&self) -> &Compiler {
        self.compiler
    }

    fn compiler_mut(&mut self) -> &mut Compiler {
        self.compiler
    }

    fn module(&self) -> &Module {
        &self.module
    }

    fn module_mut(&mut self) -> &mut Module {
        &mut self.module
    }
}
