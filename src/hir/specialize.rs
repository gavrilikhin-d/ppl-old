use std::{collections::HashMap, sync::Arc};

use super::{Class, ClassData, FunctionType, Generic, Member, Type};

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
            Type::Unknown => unreachable!("Trying to specialize not inferred type"),
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

impl Specialize for Class {
    fn specialize_with(self, mapping: &HashMap<Type, Type>) -> Self::Output {
        if !self.read().unwrap().is_generic() {
            return self;
        }

        let class = self.read().unwrap().clone();

        let generic_parameters = class
            .generic_parameters
            .iter()
            .cloned()
            .map(|p| p.specialize_with(mapping))
            .collect::<Vec<_>>();

        let members = class
            .members
            .iter()
            .map(|m| {
                Arc::new(Member {
                    ty: m.ty.clone().specialize_with(mapping),
                    ..m.as_ref().clone()
                })
            })
            .collect::<Vec<_>>();

        if generic_parameters == self.read().unwrap().generic_parameters
            && members == self.read().unwrap().members
        {
            return self;
        }

        Class::new(ClassData {
            specialization_of: class.specialization_of.clone().or(Some(self.clone())),
            generic_parameters,
            members,
            ..class
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

impl SpecializeParameters for Class {
    fn specialize_parameters(self, args: impl IntoIterator<Item = Type>) -> Self::Output {
        let mapping = HashMap::from_iter(
            (&self)
                .read()
                .unwrap()
                .generics()
                .into_iter()
                .cloned()
                .zip(args),
        );
        self.specialize_with(&mapping)
    }
}
