use std::collections::HashMap;

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

    /// Mapping of generic types
    pub generics_mapping: HashMap<Type, Type>,

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

    /// Get specialized type for generic type
    fn get_specialized(&self, generic: Type) -> Option<Type> {
        if !self.generic_parameters.contains(&generic) {
            return FindDeclaration::parent(self)
                .unwrap()
                .get_specialized(generic);
        }

        self.generics_mapping.get(&generic).cloned()
    }
}

impl AddDeclaration for GenericContext<'_> {
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        Some(self.parent as _)
    }

    fn map_generic(&mut self, generic: Type, concrete: Type) -> Option<Type> {
        if !self.generic_parameters.contains(&generic) {
            return AddDeclaration::parent_mut(self)
                .unwrap()
                .map_generic(generic, concrete);
        }

        self.generics_mapping.insert(generic, concrete)
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
