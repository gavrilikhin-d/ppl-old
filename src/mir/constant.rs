use inkwell::values::BasicValueEnum;

use crate::ir::{Context, FunctionContext, ToIR};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Constant {
    None,
    Bool(bool),
    I { bits: u32, value: u64 },
    U { bits: u32, value: u64 },
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Constant {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Constant::*;
        Some(match self {
            None => return Option::None,
            Bool(b) => context.llvm().bool_type().const_int(*b as _, false).into(),
            I { bits, value } => context
                .llvm()
                .custom_width_int_type(*bits)
                .const_int(*value, true)
                .into(),
            U { bits, value } => context
                .llvm()
                .custom_width_int_type(*bits)
                .const_int(*value, false)
                .into(),
        })
    }
}
