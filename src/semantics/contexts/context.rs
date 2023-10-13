use std::sync::Arc;

use crate::{
    compilation::Compiler,
    hir::{FunctionDeclaration, Module},
    semantics::{AddDeclaration, FindDeclaration},
};

use super::BuiltinContext;

/// Trait for various AST lowering contexts
pub trait Context: FindDeclaration + AddDeclaration {
    /// Get parent context
    fn parent(&self) -> Option<&dyn Context> {
        None
    }

    /// Get parent context
    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        None
    }

    /// Get compiler
    fn compiler(&self) -> &Compiler {
        Context::parent(self).unwrap().compiler()
    }

    /// Get compiler
    fn compiler_mut(&mut self) -> &mut Compiler {
        Context::parent_mut(self).unwrap().compiler_mut()
    }

    /// Get current module this context is for
    fn module(&self) -> &Module {
        Context::parent(self).unwrap().module()
    }

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module {
        Context::parent_mut(self).unwrap().module_mut()
    }

    /// Get current function
    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        Context::parent(self).and_then(|p| p.function())
    }

    /// Is this a context for builtin module?
    fn is_for_builtin_module(&self) -> bool {
        self.module().is_builtin
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext {
        let module = self.compiler().builtin_module().unwrap_or(self.module());
        debug_assert!(module.is_builtin);
        BuiltinContext { module }
    }
}
