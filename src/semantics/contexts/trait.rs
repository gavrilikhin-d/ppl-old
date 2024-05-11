use std::fmt::Display;

use crate::{
    hir::{Class, Function, SelfType, Trait, Type, Variable},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
};

use super::Context;

use crate::DataHolder;

/// Context for lowering body of trait
pub struct TraitContext<'p> {
    /// Trait, which is being lowered
    pub tr: Trait,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl<'p> TraitContext<'p> {
    pub fn new(tr: Trait, parent: &'p mut dyn Context) -> Self {
        Self { tr, parent }
    }

    /// Run code in this context
    pub fn run<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
    }
}

impl Display for TraitContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TraitContext:")?;
        writeln!(f, "\tfor trait: {}", self.tr.name())
    }
}

impl FindDeclarationHere for TraitContext<'_> {
    fn find_type_here(&self, name: &str) -> Option<Type> {
        if name != "Self" {
            return None;
        }

        Some(
            SelfType {
                associated_trait: self.tr.clone(),
            }
            .into(),
        )
    }

    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        self.tr
            .read()
            .unwrap()
            .functions
            .values()
            .filter(move |f| f.read().unwrap().name_parts().len() == n)
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

    fn add_type(&mut self, _ty: Class) {
        todo!("types in traits")
    }

    fn add_trait(&mut self, _tr: Trait) {
        todo!("traits in traits?")
    }

    fn add_function(&mut self, f: Function) {
        f.write().unwrap().tr = Some(self.tr.clone());
        self.tr
            .write()
            .unwrap()
            .functions
            .insert(f.name().to_string(), f);
    }

    fn add_variable(&mut self, _v: Variable) {
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
