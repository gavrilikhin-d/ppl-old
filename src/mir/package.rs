use std::cell::RefCell;

use crate::ir::{inkwell::FnType, Context, FunctionContext, ModuleContext, ToIR};

use super::{
    body::Body,
    ty::{Struct, Type},
};

use derive_more::Into;
use inkwell::{types::BasicMetadataTypeEnum, values::FunctionValue};

pub struct Package {
    pub types: Vec<Struct>,
    pub functions: Vec<FunctionData>,
}

// ID of a function in current package
#[derive(Debug, PartialEq, Eq, Clone, Copy, Into)]
pub struct Function(pub usize);

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm>> for Function {
    type IR = FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        CURRENT_PACKAGE.with_borrow(|package| {
            let f = &package.functions[self.0];
            if let Some(f) = context.functions().get(&f.name) {
                return f;
            }

            f.to_ir(context)
        })
    }
}

pub struct FunctionData {
    pub name: String,
    pub parameters: Vec<ParameterData>,
    pub return_type: Type,
    pub body: Option<Body>,
}

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm>> for FunctionData {
    type IR = FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let params: Vec<BasicMetadataTypeEnum> = self
            .parameters
            .iter()
            .filter_map(|p| p.ty.to_ir(context).try_into().ok())
            .collect();
        let return_type = self.return_type.to_ir(context);
        let ty = return_type.fn_type(&params, false);

        let f = context.module.add_function(&self.name, ty, None);

        if let Some(body) = &self.body {
            body.to_ir(&mut FunctionContext::new(context, f));
        }

        f
    }
}

pub struct ParameterData {
    pub name: String,
    pub ty: Type,
}

impl Package {
    pub fn new() -> Self {
        Self {
            types: vec![],
            functions: vec![],
        }
    }
}

thread_local! {
    pub static CURRENT_PACKAGE: RefCell<Package> = RefCell::new(Package::new());
}
