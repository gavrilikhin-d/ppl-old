use derive_visitor::VisitorMut;
use log::debug;

use crate::hir::{Call, Generic, Type};

use super::Context;

#[derive(VisitorMut)]
#[visitor(Call(exit))]
pub struct TraitFunctionsLinker<'ctx, C: Context> {
    context: &'ctx mut C,
}

impl<'ctx, C: Context> TraitFunctionsLinker<'ctx, C> {
    pub fn new(context: &'ctx mut C) -> Self {
        Self { context }
    }

    fn exit_call(&mut self, call: &mut Call) {
        let f = call.function.read().unwrap();
        // FIXME: definition may be overrided
        if f.is_generic() || !f.is_from_trait() || f.is_definition() {
            return;
        }

        debug!(target: "linking-trait-fn-from", "{f}");
        // Unknown type here is ok, because we don't have selfs any more
        let real_impl = self
            .context
            .find_implementation(&f, &Type::Unknown)
            .unwrap();
        drop(f);

        call.function = real_impl;
        debug!(target: "linking-trait-fn-to", "{}", call.function);
    }
}
