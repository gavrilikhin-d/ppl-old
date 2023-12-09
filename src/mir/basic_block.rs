use inkwell::values::InstructionValue;

use crate::ir::{FunctionContext, ToIR};

use super::{constant::Constant, local::LocalID, operand::Operand, statement::Statement};

use derive_more::Into;

#[derive(Clone)]
pub struct BasicBlock {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for BasicBlock {
    type IR = inkwell::basic_block::BasicBlock<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        for statement in &self.statements {
            statement.to_ir(context);
        }

        self.terminator.to_ir(context);

        context.builder.get_insert_block().unwrap()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Into)]
pub struct BasicBlockID(pub usize);

#[derive(Clone)]
pub enum Terminator {
    Return,
    GoTo {
        target: BasicBlockID,
    },
    Switch {
        operand: Operand,
        cases: Vec<SwitchCase>,
        default: Option<BasicBlockID>,
    },
}

#[derive(Clone)]
pub struct SwitchCase {
    pub value: Constant,
    pub target: BasicBlockID,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Terminator {
    type IR = InstructionValue<'llvm>;
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Terminator::*;
        match self {
            Return => {
                let ret = context.load(LocalID::FOR_RETURN_VALUE);

                context
                    .builder
                    .build_return(ret.as_ref().map(|ret| ret as _))
            }
            GoTo { target } => {
                let bb = context
                    .function
                    .get_basic_blocks()
                    .get(target.0 + 1)
                    .unwrap()
                    .clone();

                context.builder.build_unconditional_branch(bb)
            }
            Switch { .. } => {
                todo!()
            }
        }
    }
}
