use inkwell::values::{AnyValue, InstructionValue};

use crate::{
    ir::{FunctionContext, ToIR},
    mir::body::Body,
};

use super::{constant::Constant, local::LocalID, operand::Operand, statement::Statement};

pub struct BasicBlock {
    pub statements: Vec<Statement>,
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
    GoTo {
        target: BasicBlockID,
    },
    Switch {
        operand: Operand,
        cases: Vec<SwitchCase>,
        default: Option<BasicBlockID>,
    },
}

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
                let ret = context
                    .function
                    .get_first_basic_block()
                    .map(|bb| {
                        bb.get_first_instruction()
                            .filter(|i| {
                                i.get_name().map(|n| n.to_str().unwrap())
                                    == Some(Body::RETURN_VALUE_NAME)
                            })
                            .map(|i| i.as_any_value_enum().into_pointer_value())
                    })
                    .flatten()
                    .map(|ret| {
                        context.builder.build_load(
                            context.function.get_type().get_return_type().unwrap(),
                            ret,
                            "",
                        )
                    });

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
            Switch {
                operand,
                cases,
                default,
            } => {
                todo!()
            }
        }
    }
}
