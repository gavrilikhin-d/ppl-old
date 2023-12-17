use inkwell::values::{InstructionValue, IntValue};

use crate::ir::{FunctionContext, ToIR};

use super::{constant::Constant, local::LocalID, operand::Operand, statement::Statement};

use derive_more::Into;

pub struct BasicBlockWithID<'bb> {
    pub id: BasicBlock,
    pub data: &'bb BasicBlockData,
}

#[derive(Clone)]
pub struct BasicBlockData {
    pub statements: Vec<Statement>,
    pub terminator: Terminator,
}

impl BasicBlockData {
    pub fn successors(&self) -> impl Iterator<Item = BasicBlock> {
        self.terminator.destinations()
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for BasicBlockData {
    type IR = inkwell::basic_block::BasicBlock<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        for statement in &self.statements {
            statement.to_ir(context);
        }

        self.terminator.to_ir(context);

        context.builder.get_insert_block().unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Into)]
pub struct BasicBlock(pub usize);

#[derive(Clone)]
pub enum Terminator {
    Return,
    GoTo {
        target: BasicBlock,
    },
    Switch {
        operand: Operand,
        cases: Vec<SwitchCase>,
        default: BasicBlock,
    },
}

impl Terminator {
    /// Destinations of this terminator
    pub fn destinations(&self) -> impl Iterator<Item = BasicBlock> {
        use Terminator::*;
        match self {
            Return => vec![],
            GoTo { target } => vec![*target],
            Switch { cases, default, .. } => Vec::from_iter(
                cases
                    .iter()
                    .map(|case| case.target)
                    .chain(std::iter::once(*default)),
            ),
        }
        .into_iter()
    }
}

#[derive(Clone)]
pub struct SwitchCase {
    pub value: Constant,
    pub target: BasicBlock,
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
                let bb = context.bb(*target);

                context.builder.build_unconditional_branch(bb)
            }
            Switch {
                operand,
                cases,
                default,
            } => {
                // TODO: create tags or use any other method to match non-integer values
                let value: IntValue<'_> = operand.to_ir(context).unwrap().try_into().unwrap();

                let else_block = context.bb(*default);

                let cases: Vec<_> = cases
                    .iter()
                    .map(|case| {
                        let value: IntValue<'_> =
                            case.value.to_ir(context).unwrap().try_into().unwrap();
                        let target = context.bb(case.target);
                        (value, target)
                    })
                    .collect();

                context.builder.build_switch(value, else_block, &cases)
            }
        }
    }
}
