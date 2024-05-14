use std::{collections::HashMap, fmt::Display};

use crate::{
    hir::{FunctionData, GenericType, SelfType, Type, TypeReference, Typed},
    named::Named,
    semantics::{AddDeclaration, ConvertibleTo, FindDeclaration, FindDeclarationHere},
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
    pub fn for_fn(f: &FunctionData, parent: &'p mut impl Context) -> Self {
        let mut candidate_context = Self::for_generics(f.generic_types.clone(), parent);

        if let Some(tr) = &f.tr {
            candidate_context
                .generic_parameters
                .push(tr.self_type().into());
        }

        return candidate_context;
    }

    /// Create generic context for function
    pub fn for_fn_with_args<'a>(
        f: &FunctionData,
        args: impl IntoIterator<Item = impl Typed>,
        parent: &'p mut impl Context,
    ) -> Self {
        let mut context = Self::for_fn(f, parent);

        args.into_iter()
            .map(|a| a.ty())
            .zip(f.parameters().map(|p| p.ty()))
            .for_each(|(a, b)| {
                a.convertible_to(b).within(&mut context).unwrap();
            });

        return context;
    }

    /// Create generic context for generic parameters
    pub fn for_generics(generic_parameters: Vec<Type>, parent: &'p mut impl Context) -> Self {
        Self {
            generic_parameters,
            generics_mapping: HashMap::new(),
            parent,
        }
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

    /// Run code in this context
    pub fn run<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        f(self)
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
