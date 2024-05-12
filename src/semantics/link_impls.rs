use derive_visitor::{DriveMut, VisitorMut};
use log::debug;

use crate::{
    hir::{Call, FunctionData, Generic, ModuleData, Type},
    semantics::{GenericContext, Monomorphize},
    DataHolder,
};

use super::Context;

#[derive(VisitorMut)]
#[visitor(ModuleData(enter), Call(enter), FunctionData(enter))]
pub struct TraitFunctionsLinker<'ctx, C: Context> {
    context: &'ctx mut C,
}

impl<'ctx, C: Context> TraitFunctionsLinker<'ctx, C> {
    pub fn new(context: &'ctx mut C) -> Self {
        Self { context }
    }

    fn enter_module_data(&mut self, module: &mut ModuleData) {
        module.monomorphized_functions.drive_mut(self);
        module.functions.drive_mut(self);
    }

    fn enter_function_data(&mut self, f: &mut FunctionData) {
        if f.is_generic() || !f.is_from_trait() || f.is_definition() || f.mangled_name.is_some() {
            return;
        }

        debug!(target: "linking-trait-fn-from", "{f}");
        // Don't have selfs any more so no need to specialize it
        let mut real_impl = self
            .context
            .find_implementation(&f, None)
            .unwrap()
            .read()
            .unwrap()
            .clone();

        GenericContext::for_fn_with_args(&real_impl, f.parameters(), self.context)
            .run(|context| real_impl.monomorphize(context));
        *f = real_impl;
        debug!(target: "linking-trait-fn-to", "{f}");
    }

    fn enter_call(&mut self, call: &mut Call) {
        call.function.drive_mut(self);
    }
}
