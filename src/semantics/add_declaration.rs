use std::sync::Arc;

use crate::hir::{Function, TraitDeclaration, TypeDeclaration, VariableDeclaration};

pub trait AddDeclaration {
    /// Get parent context
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        None
    }

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        self.parent_mut().unwrap().add_type(ty)
    }

    /// Add trait to context
    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.parent_mut().unwrap().add_trait(tr)
    }

    /// Add function to context
    fn add_function(&mut self, f: Function) {
        self.parent_mut().unwrap().add_function(f)
    }

    /// Add variable to context
    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        self.parent_mut().unwrap().add_variable(v)
    }
}
