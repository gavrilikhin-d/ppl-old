use inkwell::values::BasicValueEnum;

use crate::ir::{FunctionContext, ToIR};

use super::{constant::Constant, local::LocalID};

use derive_more::From;

#[derive(PartialEq, Eq, Clone, Copy, From)]
pub enum Operand {
    Copy(LocalID),
    Move(LocalID),
    #[from]
    Constant(Constant),
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Operand {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Operand::*;
        match self {
            Copy(_local) => todo!(),
            Move(local) => context.load(*local),
            Constant(constant) => constant.to_ir(context),
        }
    }
}
