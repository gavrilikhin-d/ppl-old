use inkwell::values::InstructionValue;

use crate::ir::{FunctionContext, ToIR};

pub struct BasicBlock {
    pub terminator: Terminator,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for BasicBlock {
    type IR = inkwell::basic_block::BasicBlock<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        self.terminator.to_ir(context);

        context.builder.get_insert_block().unwrap()
    }
}

pub struct BasicBlockID(pub usize);

pub enum Terminator {
    Return,
    GoTo { target: BasicBlockID },
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Terminator {
    type IR = InstructionValue<'llvm>;
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Terminator::*;
        match self {
            Return => context.builder.build_return(None),
            GoTo { target } => {
                let bb = context
                    .function
                    .get_basic_blocks()
                    .get(target.0 + 1)
                    .unwrap()
                    .clone();

                context.builder.build_unconditional_branch(bb)
            }
        }
    }
}
