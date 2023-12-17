use inkwell::{
    types::BasicTypeEnum,
    values::{InstructionValue, PointerValue},
};

use crate::ir::{FunctionContext, ToIR};

use super::{
    local::LocalID,
    operand::Operand,
    package::{Package, CURRENT_PACKAGE},
    ty::{Field, Type},
};

#[derive(Clone)]
pub struct Place {
    pub local: LocalID,
    pub projections: Vec<Projection>,
}

impl From<LocalID> for Place {
    fn from(value: LocalID) -> Self {
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
        let base_ty = context.body.locals().nth(self.local.0).unwrap().ty;
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
pub enum Statement {
    Assign { lhs: Place, rhs: Operand },
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
