use inkwell::values::BasicValueEnum;

use crate::ir::{Context, FunctionContext, ToIR};

use super::ty::IntegerType;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Constant {
    None,
    Bool(bool),
    Integer { value: u64, ty: IntegerType },
}

impl Constant {
    pub fn i32(value: i32) -> Self {
        Constant::Integer {
            value: value as _,
            ty: IntegerType::I(32),
        }
    }
}

impl From<bool> for Constant {
    fn from(value: bool) -> Self {
        Constant::Bool(value)
    }
}

impl From<i32> for Constant {
    fn from(value: i32) -> Self {
        Constant::i32(value)
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Constant {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Constant::*;
        Some(match self {
            None => return Option::None,
            Bool(b) => context.llvm().bool_type().const_int(*b as _, false).into(),
            Integer { value, ty } => context
                .llvm()
                .custom_width_int_type(ty.bits())
                .const_int(*value, ty.signed())
                .into(),
        })
    }
}
