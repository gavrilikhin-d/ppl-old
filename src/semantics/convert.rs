use std::sync::Arc;

use crate::{hir::Type, AddSourceLocation, WithSourceLocation};

use super::{
    error::{NotConvertible, TypeMismatch, TypeWithSpan},
    FindDeclaration, Implements,
};

/// Trait to convert one type to another
pub trait Convert {
    /// Convert this type to another type
    fn convert_to(&self, ty: WithSourceLocation<Type>) -> ConversionRequest;
}

impl Convert for WithSourceLocation<Type> {
    fn convert_to(&self, to: WithSourceLocation<Type>) -> ConversionRequest {
        ConversionRequest {
            from: self.clone(),
            to,
        }
    }
}

/// Helper struct to perform check within context
pub struct ConversionRequest {
    from: WithSourceLocation<Type>,
    to: WithSourceLocation<Type>,
}

impl ConversionRequest {
    /// Convert one type to another within context
    pub fn within(&self, context: &impl FindDeclaration) -> Result<Type, NotConvertible> {
        let from = self.from.value.without_ref();
        let to = self.to.value.without_ref();
        let convertible = match (from.clone(), to.clone()) {
            (Type::Trait(tr), Type::SelfType(s)) => {
                Arc::ptr_eq(&tr, &s.associated_trait.upgrade().unwrap())
            }
            (Type::Class(c), Type::SelfType(s)) => c
                .implements(s.associated_trait.upgrade().unwrap())
                .within(context)
                .map(|_| true)?,
            (Type::Class(c), Type::Trait(tr)) => {
                c.implements(tr.clone()).within(context).map(|_| true)?
            }
            (_, Type::Generic(to)) => {
                if let Some(constraint) = to.constraint {
                    self.from
                        .convert_to(
                            constraint
                                .referenced_type
                                .clone()
                                .at(self.to.source_location.clone()),
                        )
                        .within(context)
                        .map(|_| true)?
                } else {
                    true
                }
            }
            (Type::Generic(from), _) if from.constraint.is_some() => from
                .constraint
                .unwrap()
                .referenced_type
                .at(self.from.source_location.clone())
                .convert_to(self.to.clone())
                .within(context)
                .map(|_| true)?,
            (Type::Class(from), Type::Class(to)) => {
                if to.specialization_of == Some(from.clone())
                    || from.specialization_of.is_some()
                        && to.specialization_of == from.specialization_of
                {
                    from.generics()
                        .iter()
                        .zip(to.generics().iter())
                        .all(|(from, to)| {
                            from.clone()
                                .at(self.from.source_location.clone())
                                .convert_to(to.clone().at(self.to.source_location.clone()))
                                .within(context)
                                // TODO: Add error
                                .is_ok()
                        })
                } else {
                    from == to
                }
            }
            (from, to) => from == to,
        };

        if !convertible {
            return Err(TypeMismatch {
                // TODO: use WithSourceLocation for TypeWithSpan
                got: TypeWithSpan {
                    ty: self.from.value.clone(),
                    at: self.from.source_location.at.clone(),
                    source_file: self.from.source_location.source_file.clone(),
                },
                expected: TypeWithSpan {
                    ty: self.to.value.clone(),
                    at: self.to.source_location.at.clone(),
                    source_file: self.to.source_location.source_file.clone(),
                },
            }
            .into());
        }

        Ok(to)
    }
}
