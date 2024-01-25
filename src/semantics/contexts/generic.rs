use std::{collections::HashMap, fmt::Display};

use crate::{
    hir::{Function, GenericType, Type, TypeReference, Typed},
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

impl<'p> GenericContext<'p> {
    /// Create generic context for function
    pub fn for_fn(f: Function, parent: &'p mut impl Context) -> Self {
        let mut candidate_context = GenericContext {
            generic_parameters: f.read().unwrap().generic_types.clone(),
            generics_mapping: HashMap::new(),
            parent,
        };

        if let Some(ty) = f
            .read()
            .unwrap()
            .parameters()
            .map(|p| p.ty())
            .find(|ty| matches!(ty, Type::SelfType(_)))
        {
            candidate_context.generic_parameters.push(ty);
        }

        return candidate_context;
    }

    /// Generate a new unique name for generic parameter
    pub fn new_unique_name(&mut self) -> String {
        const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        assert!(self.generic_parameters.len() < ALPHABET.len());

        let mut i = ALPHABET.find('T').unwrap();
        let mut name = &ALPHABET[i..i + 1];
        while self.generic_parameters.iter().any(|p| p.name() == name) {
            i = (i + 1) % ALPHABET.len();
            name = &ALPHABET[i..i + 1];
        }
        name.to_string()
    }
}

impl Display for GenericContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "GenericContext:")?;
        writeln!(
            f,
            "\tfor types: [{}]",
            self.generic_parameters
                .iter()
                .map(|p| p.name())
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if !self.generics_mapping.is_empty() {
            writeln!(
                f,
                "\tmappings: {:?}",
                self.generics_mapping
                    .iter()
                    .map(|(k, v)| format!("{k} -> {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )?;
        }
        Ok(())
    }
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

    fn new_generic_for_trait(&mut self, ty: TypeReference) -> GenericType {
        let generic = GenericType {
            name: self.new_unique_name().into(),
            generated: true,
            constraint: Some(ty),
        };
        self.generic_parameters.push(generic.clone().into());
        generic
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
