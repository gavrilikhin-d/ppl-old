use inkwell::{
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValueEnum, InstructionValue, PointerValue},
};

use crate::ir::{FunctionContext, ToIR};

use super::{
    local::Local,
    operand::Operand,
    package::{Function, Package, CURRENT_PACKAGE},
    ty::{Field, Type},
};

#[derive(Clone)]
pub struct Place {
    pub local: Local,
    pub projections: Vec<Projection>,
}

impl From<Local> for Place {
    fn from(value: Local) -> Self {
        Place {
            local: value,
            projections: vec![],
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Place {
    type IR = Option<PointerValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let mut base = self.local.to_ir(context);
        let base_ty = context.body.locals().nth(self.local.index()).unwrap().ty;
        let mut base_ty: BasicTypeEnum = base_ty.to_ir(context).try_into().unwrap();

        let mut proj = self.projections.iter();
        while let Some((ptr, proj)) = base.zip(proj.next()) {
            use Projection::*;
            base = match proj {
                Field { index, ty } => {
                    let ty: Option<BasicTypeEnum> = ty.to_ir(context).try_into().ok();
                    ty.map(|ty| {
                        let res = context
                            .builder
                            .build_struct_gep(base_ty, ptr, *index as u32, "")
                            .unwrap()
                            .into();
                        base_ty = ty;
                        res
                    })
                }
            }
        }

        base
    }
}

#[derive(Clone)]
pub enum Projection {
    Field { index: usize, ty: Type },
}

#[derive(Clone)]
pub enum RValue {
    Operand(Operand),
    Call {
        function: Function,
        args: Vec<Operand>,
    },
}

impl<T: Into<Operand>> From<T> for RValue {
    fn from(value: T) -> Self {
        Self::Operand(value.into())
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for RValue {
    type IR = Option<BasicValueEnum<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use RValue::*;
        match self {
            Operand(op) => op.to_ir(context),
            Call { function, args } => {
                let f = function.to_ir(context.module_context);
                let args: Vec<BasicMetadataValueEnum> = args
                    .iter()
                    .filter_map(|arg| arg.to_ir(context).map(|arg| arg.into()))
                    .collect();
                context
                    .builder
                    .build_call(f, &args, "")
                    .try_as_basic_value()
                    .left()
            }
        }
    }
}

#[derive(Clone)]
pub enum Statement {
    Assign { lhs: Place, rhs: RValue },
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Statement {
    type IR = Option<InstructionValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        use Statement::*;
        Some(match self {
            Assign { lhs, rhs } => {
                let lhs = lhs.to_ir(context);
                let rhs = rhs.to_ir(context);
                if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
                    context.builder.build_store(lhs, rhs)
                } else {
                    return None;
                }
            }
        })
    }
}
