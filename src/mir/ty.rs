use crate::ir::{Context, ToIR};

use derive_more::{From, Into};
use inkwell::{types::BasicTypeEnum, AddressSpace};

use super::package::CURRENT_PACKAGE;

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

#[derive(PartialEq, Eq, Clone, From)]
pub enum Type {
    None,
    Bool,
    I(u32),
    U(u32),
    Pointer(Box<Type>),
    #[from]
    Struct(StructID),
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
            Pointer(_) => {
                // LLVM makes no difference between pointers
                llvm.i8_type().ptr_type(AddressSpace::default()).into()
            }
            Struct(s) => s.to_ir(context).into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Into)]
pub struct StructID(pub usize);

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for StructID {
    type IR = inkwell::types::StructType<'llvm>;

    fn to_ir(&self, context: &mut C) -> Self::IR {
        CURRENT_PACKAGE.with_borrow(|package| {
            let ty = &package.types[self.0];
            if let Some(ty) = context.llvm().get_struct_type(&ty.name) {
                return ty;
            }

            ty.to_ir(context)
        })
    }
}

pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for Struct {
    type IR = inkwell::types::StructType<'llvm>;

    fn to_ir(&self, context: &mut C) -> Self::IR {
        let ty = context.llvm().opaque_struct_type(&self.name);
        ty.set_body(
            self.fields
                .iter()
                .filter_map(|f| f.ty.to_ir(context).try_into().ok())
                .collect::<Vec<_>>()
                .as_slice(),
            false,
        );
        ty
    }
}

pub struct Field {
    pub name: String,
    pub ty: Type,
}
