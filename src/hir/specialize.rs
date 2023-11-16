use std::{collections::HashMap, sync::Arc};

use super::{FunctionType, Generic, Type, TypeDeclaration};

/// Specialize type using given mapping
pub trait Specialize: Generics
where
    Self: Sized,
{
    type Output = Self;

    /// Specialize type using
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output;

    fn specialize_by_order(self, args: impl IntoIterator<Item = Type>) -> Self::Output {
        self.specialize_with(&HashMap::from_iter(
            self.generics().into_iter().cloned().zip(args),
        ))
    }
}

impl Specialize for Type {
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output {
        match self {
            Type::Class(c) => c.specialize_with(mapping).into(),
            Type::Function(f) => f.specialize_with(mapping).into(),
            Type::Trait(_) | Type::SelfType(_) | Type::Generic(_) => {
                if let Some(ty) = mapping.get(&self) {
                    ty.clone()
                } else {
                    self
                }
            }
        }
    }
}

impl Specialize for Arc<TypeDeclaration> {
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output {
        if !self.is_generic() {
            return self;
        }

        let generic_parameters = self
            .generic_parameters
            .iter()
            .map(|p| p.specialize_with(mapping))
            .collect::<Vec<_>>();

        let members = self
            .members
            .into_iter()
            .map(|mut m| {
                m.ty = m.ty.specialize_with(mapping);
                m
            })
            .collect::<Vec<_>>();

        if generic_parameters == self.generic_parameters && members == self.members {
            return self;
        }

        Arc::new(TypeDeclaration {
            specialization_of: self.specialization_of.or(Some(self.clone())),
            generic_parameters,
            members,
            ..self.as_ref().clone()
        })
    }
}

impl Specialize for FunctionType {
    fn specialize_with(mut self, mapping: &HashMap<Type, Type>) -> Self::Output {
        FunctionType::build()
            .with_parameters(
                self.parameters
                    .into_iter()
                    .map(|p| p.specialize_with(mapping))
                    .collect(),
            )
            .with_return_type(self.return_type.specialize_with(mapping))
    }
}
