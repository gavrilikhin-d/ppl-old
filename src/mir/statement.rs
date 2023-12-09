use inkwell::values::InstructionValue;

use crate::ir::{FunctionContext, ToIR};

use super::{local::LocalID, operand::Operand};

#[derive(Clone)]
pub enum Statement {
    Assign { lhs: LocalID, rhs: Operand },
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Statement {
    type IR = Option<InstructionValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Statement::*;
        Some(match self {
            Assign { lhs, rhs } => {
                let lhs = lhs.to_ir(context);
                let rhs = rhs.to_ir(context);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    context.builder.build_store(lhs, rhs)
                } else {
                    return None;
                }
            }
        })
    }
}
