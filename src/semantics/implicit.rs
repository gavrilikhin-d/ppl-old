use crate::hir::{Expression, ImplicitConversion, ImplicitConversionKind::*, Typed};

use super::Context;

/// Trait to wrap expression with implicit dereference/reference
pub trait Implicit {
    /// Implicitly dereference this expression
    fn dereference(self) -> Self;

    /// Implicitly reference this expression
    fn reference(self, context: &impl Context) -> Self;

    /// Implicitly reference this with mutable reference
    fn reference_mut(self, context: &impl Context) -> Self;

    /// Implicitly copy this expression
    fn copy(self) -> Self;
}

impl Implicit for Expression {
    fn dereference(self) -> Self {
        if !self.ty().is_any_reference() {
            return self;
        }

        ImplicitConversion {
            kind: Dereference,
            ty: self.ty().without_ref(),
            expression: Box::new(self),
        }
        .into()
    }

    fn reference(self, context: &impl Context) -> Self {
        ImplicitConversion {
            kind: Reference,
            ty: context.builtin().types().reference_to(self.ty()),
            expression: Box::new(self),
        }
        .into()
    }

    fn reference_mut(self, context: &impl Context) -> Self {
        ImplicitConversion {
            kind: Reference,
            ty: context.builtin().types().reference_mut_to(self.ty()),
            expression: Box::new(self),
        }
        .into()
    }

    fn copy(self) -> Self {
        ImplicitConversion {
            kind: Copy,
            ty: self.ty().clone(),
            expression: Box::new(self),
        }
        .into()
    }
}
