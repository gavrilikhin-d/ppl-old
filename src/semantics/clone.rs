use crate::{
    hir::{Call, Expression, Typed},
    syntax::Ranged,
};

use super::Context;

pub trait CloneIfNeeded {
    fn clone_if_needed(self, context: &mut impl Context) -> Expression;
}

impl CloneIfNeeded for Expression {
    fn clone_if_needed(self, context: &mut impl Context) -> Expression {
        if !matches!(
            self,
            Expression::VariableReference(_) | Expression::MemberReference(_)
        ) {
            return self;
        }

        if let Some(clone) = context.clone_for(self.ty()) {
            return Call {
                range: self.range(),
                function: clone,
                generic: None,
                // FIXME: create temporary variable,
                // if it's complex expr
                args: vec![self],
            }
            .into();
        }

        return self;
    }
}
