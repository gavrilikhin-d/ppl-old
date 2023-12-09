use crate::ir::{Context, ToIR};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Type {
    None,
    Bool,
    I(u32),
    U(u32),
}

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for Type {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    fn to_ir(&self, context: &mut C) -> Self::IR {
        let llvm = context.llvm();
        use Type::*;
        match self {
            None => llvm.void_type().into(),
            Bool => llvm.bool_type().into(),
            I(bits) | U(bits) => llvm.custom_width_int_type(*bits).into(),
        }
    }
}
