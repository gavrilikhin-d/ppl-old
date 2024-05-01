use derive_visitor::VisitorMut;

use crate::{
    hir::{
        Assignment, Call, Expression, ImplicitConversion, ImplicitConversionKind, Initializer,
        Return, Typed, VariableData,
    },
    syntax::Ranged,
};

use super::Context;

#[derive(VisitorMut)]
#[visitor(
    Assignment(exit),
    Return(exit),
    Initializer(exit),
    Call(exit),
    VariableData(exit)
)]
pub struct Clonner<'ctx> {
    context: &'ctx mut dyn Context,
}

impl<'ctx> Clonner<'ctx> {
    pub fn new(context: &'ctx mut dyn Context) -> Self {
        Self { context }
    }

    fn clone_expr(&mut self, expr: &mut Expression) -> Option<()> {
        if !matches!(
            expr,
            Expression::VariableReference(_)
                | Expression::MemberReference(_)
                | Expression::ImplicitConversion(ImplicitConversion {
                    kind: ImplicitConversionKind::Dereference,
                    ..
                })
        ) {
            return None;
        }

        let clone = self.context.clone_for(expr.ty())?;
        let mut expr_new: Expression = Call {
            range: expr.range(),
            function: clone,
            generic: None,
            args: vec![],
        }
        .into();
        std::mem::swap(&mut expr_new, expr);
        match expr {
            Expression::Call(call) => {
                call.args.push(expr_new);
            }
            _ => unreachable!("We've just replaced self with call"),
        }
        Some(())
    }

    fn exit_variable_data(&mut self, var: &mut VariableData) {
        var.initializer.as_mut().map(|expr| self.clone_expr(expr));
    }

    fn exit_assignment(&mut self, assignment: &mut Assignment) {
        self.clone_expr(&mut assignment.value);
    }

    fn exit_return(&mut self, ret: &mut Return) {
        ret.value_mut().map(|expr| self.clone_expr(expr));
    }

    fn exit_initializer(&mut self, init: &mut Initializer) {
        self.clone_expr(&mut init.value);
    }

    fn exit_call(&mut self, call: &mut Call) {
        for arg in &mut call.args {
            self.clone_expr(arg);
        }
    }
}
