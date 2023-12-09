use std::borrow::Cow;

use inkwell::{types::BasicTypeEnum, values::PointerValue};

use crate::{
    ir::{FunctionContext, ToIR},
    named::Named,
};

use super::ty::Type;

use derive_more::Into;

pub struct Local {
    pub ty: Type,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Local {
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
#[derive(PartialEq, Eq, Clone, Copy, Into)]
pub struct LocalID(pub usize);

impl LocalID {
    /// Local ID for the return value of a function.
    pub const FOR_RETURN_VALUE: Self = Self(0);
}

impl Named for LocalID {
    fn name(&self) -> Cow<'_, str> {
        format!("_{i}", i = self.0).into()
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for LocalID {
    type IR = Option<PointerValue<'llvm>>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        context.locals.get(self.0).unwrap().clone()
    }
}
