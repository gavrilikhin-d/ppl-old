use crate::{
    hir::{Call, Expression, Typed},
    syntax::Ranged,
};

use super::Context;

pub trait CloneIfNeeded: Sized {
    fn clone_if_needed_inplace(&mut self, context: &mut impl Context);

    fn clone_if_needed(mut self, context: &mut impl Context) -> Self {
        self.clone_if_needed_inplace(context);
        self
    }
}

impl CloneIfNeeded for Expression {
    fn clone_if_needed_inplace(&mut self, context: &mut impl Context) {
        if !matches!(
            self,
            Expression::VariableReference(_) | Expression::MemberReference(_)
        ) {
            return;
        }

        if let Some(clone) = context.clone_for(self.ty()) {
            let mut expr: Expression = Call {
                range: self.range(),
                function: clone,
                generic: None,
                args: vec![],
            }
            .into();
            std::mem::swap(&mut expr, self);
            match self {
                Expression::Call(call) => {
                    call.args.push(expr);
                }
                _ => unreachable!("We've just replaced self with call"),
            }
        }
    }
}
