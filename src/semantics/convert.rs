use crate::{
    hir::{Class, Expression, FunctionType, GenericType, SelfType, Trait, Type, Typed},
    mutability::Mutable,
    semantics::error::ReferenceMutToImmutable,
    syntax::Ranged,
    SourceLocation, WithSourceLocation,
};

use super::{
    clone::CloneIfNeeded,
    error::{NotConvertible, NotImplemented, TypeMismatch, TypeWithSpan},
    Context, Implements, Implicit,
};

/// Trait to check if one type is convertible to another
pub trait ConvertibleTo
where
    Self: Sized,
{
    /// Is this type convertible to another type?
    fn convertible_to(&self, to: Type) -> ConvertibleToRequest<'_, Self> {
        ConvertibleToRequest { from: self, to }
    }
}

/// Helper struct to perform check within context
pub struct ConvertibleToRequest<'s, S> {
    from: &'s S,
    to: Type,
}

impl ConvertibleTo for Type {}
impl ConvertibleToRequest<'_, Type> {
    /// Check if one type can be converted to another type within context
    pub fn within(self, context: &mut impl Context) -> Result<bool, NotImplemented> {
        let from = self.from.without_ref();
        let to = self.to.without_ref();

        if let Some(specialized) = context.get_specialized(from.clone()) {
            return specialized.convertible_to(to).within(context);
        }

        if let Some(specialized) = context.get_specialized(to.clone()) {
            return from.convertible_to(specialized).within(context);
        }

        match from {
            Type::Unknown => unreachable!(
                "Trying to check if not inferred type is convertible to some other type"
            ),
            Type::Class(c) => c.convertible_to(to).within(context),
            Type::Function(f) => f.convertible_to(to).within(context),
            Type::Generic(g) => g.convertible_to(to).within(context),
            Type::SelfType(s) => s.convertible_to(to).within(context),
            Type::Trait(tr) => tr.convertible_to(to).within(context),
        }
    }
}

impl ConvertibleTo for Class {}
impl ConvertibleToRequest<'_, Class> {
    /// Check if struct type can be converted to another type within context
    pub fn within(self, context: &mut impl Context) -> Result<bool, NotImplemented> {
        let from = self.from;
        let to = self.to;
        Ok(match to {
            Type::Class(to) => {
                let to = to.read().unwrap();
                if to.specialization_of == Some(self.from.clone())
                    || from.read().unwrap().specialization_of.is_some()
                        && to.specialization_of == from.read().unwrap().specialization_of
                {
                    from.read()
                        .unwrap()
                        .generics()
                        .iter()
                        .zip(to.generics().iter())
                        .all(|(from, to)| {
                            from.clone()
                                .convertible_to(to.clone())
                                .within(context)
                                // TODO: Add error
                                .is_ok_and(|convertible| convertible)
                        })
                } else {
                    *from.read().unwrap() == *to
                }
            }
            Type::Trait(tr) => from.implements(tr.clone()).within(context).map(|_| true)?,
            Type::SelfType(s) => {
                let convertible = from
                    .implements(s.associated_trait.clone())
                    .within(context)
                    .map(|_| true)?;
                if convertible {
                    context.map_generic(s.clone().into(), from.clone().into());
                }
                convertible
            }
            Type::Generic(g) => {
                let convertible = if let Some(constraint) = &g.constraint {
                    from.convertible_to(constraint.referenced_type.clone())
                        .within(context)?
                } else {
                    true
                };
                if convertible {
                    context.map_generic(g.clone().into(), from.clone().into());
                }
                convertible
            }
            Type::Function(_) => false,
            Type::Unknown => true,
        })
    }
}

// TODO: unify `fn <:Trait>` with `fn<T: Trait> <x: T>`
impl ConvertibleTo for Trait {}
impl ConvertibleToRequest<'_, Trait> {
    /// Check if trait can be converted to another type within context
    pub fn within(self, context: &mut impl Context) -> Result<bool, NotImplemented> {
        let from = self.from;
        let to = self.to;
        Ok(match to {
            Type::Unknown => true,
            Type::Class(_) => false,
            Type::Function(_) => false,
            Type::Generic(g) => {
                if let Some(constraint) = g.constraint {
                    from.convertible_to(constraint.referenced_type.clone())
                        .within(context)?
                } else {
                    true
                }
            }
            Type::Trait(tr) => {
                if *from == tr {
                    return Ok(true);
                }

                if from.read().unwrap().supertraits.is_empty() {
                    return Ok(false);
                }

                let res: Vec<_> = from
                    .read()
                    .unwrap()
                    .supertraits
                    .iter()
                    .cloned()
                    .map(|s| s.convertible_to(tr.clone().into()).within(context))
                    .collect();
                if res
                    .iter()
                    .find(|r| r.as_ref().is_ok_and(|res| *res))
                    .is_some()
                {
                    return Ok(true);
                }
                return res.into_iter().next().unwrap();
            }
            Type::SelfType(s) => from
                .convertible_to(s.associated_trait.into())
                .within(context)?,
        })
    }
}

impl ConvertibleTo for GenericType {}
impl ConvertibleToRequest<'_, GenericType> {
    /// Check if generic type can be converted to another type within context
    pub fn within(self, context: &mut impl Context) -> Result<bool, NotImplemented> {
        let from = self.from;
        let to = self.to;
        Ok(match to {
            Type::Unknown => true,
            Type::Class(_) => false,
            Type::Function(_) => false,
            Type::SelfType(SelfType {
                associated_trait: tr,
            })
            | Type::Trait(tr) => {
                if let Some(constraint) = &from.constraint {
                    constraint
                        .referenced_type
                        .convertible_to(tr.into())
                        .within(context)?
                } else {
                    let source_file = tr
                        .read()
                        .unwrap()
                        .module
                        .data(context.compiler())
                        .source_file()
                        .clone();
                    return Err(NotImplemented {
                        ty: from.clone().into(),
                        tr,
                        unimplemented: vec![],
                        source_file,
                    }
                    .into());
                }
            }
            Type::Generic(g) => {
                if let Some(constraint) = &from.constraint {
                    constraint
                        .referenced_type
                        .convertible_to(g.into())
                        .within(context)?
                } else {
                    g.constraint.is_none()
                }
            }
        })
    }
}

impl ConvertibleTo for FunctionType {}
impl ConvertibleToRequest<'_, FunctionType> {
    /// Check if function type can be converted to another type within context
    pub fn within(self, _context: &mut impl Context) -> Result<bool, NotImplemented> {
        let _from = self.from;
        let to = self.to;
        Ok(match to {
            Type::Class(_) => false,
            Type::Function(_) => todo!(),
            Type::Generic(_) => false,
            Type::Trait(_) => false,
            Type::SelfType(_) => false,
            Type::Unknown => true,
        })
    }
}

impl ConvertibleTo for SelfType {}
impl ConvertibleToRequest<'_, SelfType> {
    /// Check if self type can be converted to another type within context
    pub fn within(self, context: &mut impl Context) -> Result<bool, NotImplemented> {
        let from = self.from;
        let to = self.to;
        from.associated_trait.convertible_to(to).within(context)
    }
}

/// Trait to convert one type to another
pub trait Convert {
    /// Convert this type to another type
    fn convert_to(&self, ty: WithSourceLocation<Type>) -> ConversionRequest;
}

impl Convert for WithSourceLocation<Expression> {
    fn convert_to(&self, to: WithSourceLocation<Type>) -> ConversionRequest {
        ConversionRequest {
            from: self.clone(),
            to,
        }
    }
}

impl Convert for Expression {
    fn convert_to(&self, ty: WithSourceLocation<Type>) -> ConversionRequest {
        WithSourceLocation {
            value: self.clone(),
            source_location: SourceLocation {
                source_file: None,
                at: self.range().into(),
            },
        }
        .convert_to(ty)
    }
}

/// Helper struct to perform check within context
pub struct ConversionRequest {
    from: WithSourceLocation<Expression>,
    to: WithSourceLocation<Type>,
}

impl ConversionRequest {
    /// Convert one type to another within context
    pub fn within(self, context: &mut impl Context) -> Result<Expression, NotConvertible> {
        let from = self.from.value.ty();
        let to = self.to.value;

        let convertible = from.convertible_to(to.clone()).within(context)?;

        if !convertible {
            return Err(TypeMismatch {
                // TODO: use WithSourceLocation for TypeWithSpan
                got: TypeWithSpan {
                    ty: from,
                    at: self.from.source_location.at.clone(),
                    source_file: self.from.source_location.source_file.clone(),
                },
                expected: TypeWithSpan {
                    ty: to,
                    at: self.to.source_location.at,
                    source_file: self.to.source_location.source_file,
                },
            }
            .into());
        }

        if self.from.value.is_immutable() && to.is_mutable() {
            return Err(ReferenceMutToImmutable {
                at: self.from.value.range().into(),
            }
            .into());
        }

        if from.is_any_reference() && to.is_any_reference() {
            return Ok(self.from.value);
        }

        if !from.is_any_reference() && !to.is_any_reference() {
            return Ok(self.from.value.clone_if_needed(context));
        }

        if from.is_any_reference() {
            return Ok(self.from.value.dereference());
        }

        // to.is_any_reference() == true
        Ok(if to.is_immutable() {
            self.from.value.reference(context)
        } else {
            self.from.value.reference_mut(context)
        })
    }
}
