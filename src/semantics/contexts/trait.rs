use std::sync::{Arc, Weak};

use crate::{
    hir::{Function, SelfType, TraitDeclaration, Type, TypeDeclaration, VariableDeclaration},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
};

use super::Context;

/// Context for lowering body of trait
pub struct TraitContext<'p> {
    /// Trait, which is being lowered
    pub tr: TraitDeclaration,

    /// Uninitialized weak pointer to trait
    pub trait_weak: Weak<TraitDeclaration>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl FindDeclarationHere for TraitContext<'_> {
    fn find_type_here(&self, name: &str) -> Option<Type> {
        if name != "Self" {
            return None;
        }

        Some(
            SelfType {
                associated_trait: self.trait_weak.clone(),
            }
            .into(),
        )
    }

    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        self.tr
            .functions
            .values()
            .filter(move |f| f.name_parts().len() == n)
            .cloned()
            .collect()
    }
}

impl FindDeclaration for TraitContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        Some(self.parent as _)
    }
}

impl AddDeclaration for TraitContext<'_> {
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        Some(self.parent as _)
    }

    fn add_type(&mut self, _ty: Arc<TypeDeclaration>) {
        todo!("types in traits")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("traits in traits?")
    }

    fn add_function(&mut self, f: Function) {
        self.tr.functions.insert(f.name().to_string(), f);
    }

    fn add_variable(&mut self, _v: Arc<VariableDeclaration>) {
        todo!("variables in traits")
    }
}

impl Context for TraitContext<'_> {
    fn parent(&self) -> Option<&dyn Context> {
        Some(self.parent)
    }

    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        Some(self.parent)
    }
}
