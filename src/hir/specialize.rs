use std::{collections::HashMap, sync::Arc};

use super::{ClassDeclaration, FunctionType, Generic, Member, Type};

/// Specialize type using given mapping
pub trait Specialize
where
    Self: Sized,
{
    type Output = Self;

    /// Specialize type using
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output;
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

impl Specialize for Arc<ClassDeclaration> {
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output {
        if !self.is_generic() {
            return self;
        }

        let generic_parameters = self
            .generic_parameters
            .iter()
            .cloned()
            .map(|p| p.specialize_with(mapping))
            .collect::<Vec<_>>();

        let members = self
            .members
            .iter()
            .map(|m| {
                Arc::new(Member {
                    ty: m.ty.clone().specialize_with(mapping),
                    ..m.as_ref().clone()
                })
            })
            .collect::<Vec<_>>();

        if generic_parameters == self.generic_parameters && members == self.members {
            return self;
        }

        Arc::new(ClassDeclaration {
            specialization_of: self.specialization_of.clone().or(Some(self.clone())),
            generic_parameters,
            members,
            ..self.as_ref().clone()
        })
    }
}

impl Specialize for FunctionType {
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output {
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

/// Specialize class without passing explicit mapping.
/// Order of parameters will be used instead.
///
/// # Example
/// `Point<T, U>` specialize by order with [`A`, `B`] is `Point<A, B>`
pub trait SpecializeParameters
where
    Self: Sized,
{
    type Output = Self;

    /// Specialize class without passing explicit mapping.
    /// Order of parameters will be used instead.
    ///
    /// # Example
    /// `Point<T, U>` specialize by order with [`A`, `B`] is `Point<A, B>`
    fn specialize_parameters(self, args: impl IntoIterator<Item = Type>) -> Self::Output;
}

impl SpecializeParameters for Arc<ClassDeclaration> {
    fn specialize_parameters(self, args: impl IntoIterator<Item = Type>) -> Self::Output {
        let mapping = HashMap::from_iter((&self).generics().into_iter().cloned().zip(args));
        self.specialize_with(&mapping)
    }
}
