use crate::ir::{Context, ToIR};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum IntegerType {
    I(u32),
    U(u32),
}

impl IntegerType {
    /// Number of bits in this integer type
    pub fn bits(&self) -> u32 {
        use IntegerType::*;
        match self {
            I(bits) | U(bits) => *bits,
        }
    }

    /// Is this a signed integer type?
    pub fn signed(&self) -> bool {
        matches!(self, IntegerType::I(_))
    }
}

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for IntegerType {
    type IR = inkwell::types::IntType<'llvm>;

    fn to_ir(&self, context: &mut C) -> Self::IR {
        use IntegerType::*;
        match self {
            I(bits) | U(bits) => context.llvm().custom_width_int_type(*bits).into(),
        }
    }
}

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
