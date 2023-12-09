use inkwell::values::BasicValueEnum;

use crate::ir::{Context, FunctionContext, ToIR};

pub enum Constant {
    None,
    Bool(bool),
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Constant {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Constant::*;
        Some(match self {
            None => return Option::None,
            Bool(b) => context.llvm().bool_type().const_int(*b as _, false).into(),
        })
    }
}
