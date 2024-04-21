use std::fmt::Display;

use crate::{
    ast,
    compilation::Compiler,
    hir::{Class, Function, ModuleData, Trait, Variable},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, ToHIR},
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
        writeln!(f, "\tfor module: {}", self.module.name())?;
        writeln!(f, "\tvariables:")?;
        for v in self.module.variables.values() {
            writeln!(f, "{v}")?;
        }
        writeln!(f, "\ttypes:")?;
        for ty in self.module.types.values() {
            writeln!(f, "{ty}")?;
        }
        writeln!(f, "\tfunctions:")?;
        for fun in self.module.iter_functions() {
            writeln!(f, "{fun}")?;
        }
        Ok(())
    }
}

impl<'c> ModuleContext<'c> {
    pub fn new(module: ModuleData, compiler: &'c mut Compiler) -> Self {
        let mut context = Self { module, compiler };
        if context.compiler.import_builtin {
            ast::Use::builtin_module().to_hir(&mut context).unwrap();
        }
        context
    }
}

impl AsRef<ModuleData> for ModuleContext<'_> {
    fn as_ref(&self) -> &ModuleData {
        &self.module
    }
}

impl FindDeclaration for ModuleContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        None
    }
}

impl AddDeclaration for ModuleContext<'_> {
    fn add_type(&mut self, ty: Class) {
        self.module.add_type(ty)
    }

    fn add_trait(&mut self, tr: Trait) {
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
