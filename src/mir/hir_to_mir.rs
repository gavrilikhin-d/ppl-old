use crate::{hir, named::Named};

use super::{Call, Callee, Expression, MemberReference, Scope, Variable, VariableReference};

/// Context for converting HIR to MIR
pub struct Context {
    /// Current scope
    pub scope: Scope,
}

impl Context {
    /// Create a new temporary variable
    pub fn tmp_var(&mut self, value: Expression) -> VariableReference {
        let name = format!("_{}", self.scope.variables.len());
        self.var(name, value)
    }

    /// Create a new named variable
    pub fn var(&mut self, name: String, value: Expression) -> VariableReference {
        let var = Variable {
            name: name.clone(),
            value,
        };
        self.scope.variables.push(var);
        VariableReference { name }
    }
}

/// Trait for converting HIR to MIR
pub trait HIRtoMIR {
    type MIR;

    /// Convert this HIR to MIR
    fn to_mir(&self, context: &mut Context) -> Self::MIR;
}

impl HIRtoMIR for hir::Literal {
    type MIR = hir::Literal;

    fn to_mir(&self, _: &mut Context) -> Self::MIR {
        self.clone()
    }
}

impl HIRtoMIR for hir::VariableReference {
    type MIR = VariableReference;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        VariableReference {
            name: self.variable.name().to_string(),
        }
    }
}

impl HIRtoMIR for hir::MemberReference {
    type MIR = VariableReference;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        let variable = match self.base.as_ref() {
            hir::Expression::VariableReference(var) => var.to_mir(context),
            hir::Expression::MemberReference(m) => m.to_mir(context),
            _ => {
                let value = self.base.to_mir(context);
                context.tmp_var(value)
            }
        };

        let value = MemberReference {
            variable,
            member: self.member.name().to_string(),
        };
        context.tmp_var(value.into())
    }
}

impl HIRtoMIR for hir::Constructor {
    type MIR = Expression;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        self.initializers.iter().map(f)
        todo!()
    }
}

impl HIRtoMIR for hir::TypeReference {
    type MIR = Expression;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        unreachable!("type ref should be replaced with constructor in HIR")
    }
}

impl HIRtoMIR for hir::Call {
    type MIR = Expression;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        let args = self.args.into_iter().map(|arg| arg.to_mir(context));
        Call {
            callee: Callee::Function(self.function.mangled_name().to_string()),
            arguments: todo!(),
        }
        .into()
    }
}

impl HIRtoMIR for hir::Expression {
    type MIR = Expression;

    fn to_mir(&self, context: &mut Context) -> Self::MIR {
        todo!()
    }
}
