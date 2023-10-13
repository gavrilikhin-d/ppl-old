use crate::{
    hir::Type,
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
};

use super::Context;

/// Context for introducing generic parameters
pub struct GenericContext<'p> {
    /// Types of generic parameters
    pub generic_parameters: Vec<Type>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl FindDeclarationHere for GenericContext<'_> {
    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.generic_parameters
            .iter()
            .find(|p| p.name() == name)
            .cloned()
    }
}

impl FindDeclaration for GenericContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        Some(self.parent as _)
    }
}

impl AddDeclaration for GenericContext<'_> {
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        Some(self.parent as _)
    }
}

impl Context for GenericContext<'_> {
    fn parent(&self) -> Option<&dyn Context> {
        Some(self.parent)
    }

    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        Some(self.parent)
    }
}
