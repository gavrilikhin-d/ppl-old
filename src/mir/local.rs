use std::borrow::Cow;

use inkwell::{types::BasicTypeEnum, values::PointerValue};

use crate::{
    ir::{FunctionContext, ToIR},
    named::Named,
};

use super::ty::Type;

use derive_more::Into;

#[derive(Clone)]
pub struct LocalData {
    pub ty: Type,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for LocalData {
    type IR = Option<PointerValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let ty = self.ty.to_ir(context);

        let local = BasicTypeEnum::try_from(ty)
            .map(|ty| {
                let i = context.locals.len();
                let name = format!("_{i}");
                context.builder.build_alloca(ty, &name)
            })
            .ok();

        context.locals.push(local.clone());

        local
    }
}

/// ID of a local variable
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Local {
    ReturnValue,
    ArgOrVariable(usize),
}

impl Local {
    pub fn index(&self) -> usize {
        use Local::*;
        match self {
            ReturnValue => 0,
            ArgOrVariable(i) => 1 + i,
        }
    }
}

impl Named for Local {
    fn name(&self) -> Cow<'_, str> {
        format!("_{i}", i = self.index()).into()
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Local {
    type IR = Option<PointerValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        context.locals.get(self.index()).unwrap().clone()
    }
}
