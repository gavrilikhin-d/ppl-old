use derive_visitor::VisitorMut;

use crate::{
    hir::{Function, Parameter},
    DataHolder,
};

#[derive(VisitorMut)]
#[visitor(Function(enter), Parameter(enter))]
pub struct ParameterNamer {
    index: usize,
}

impl ParameterNamer {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn enter_function(&mut self, _function: &mut Function) {
        self.index = 0;
    }

    pub fn enter_parameter(&mut self, parameter: &mut Parameter) {
        let name = &mut parameter.write().unwrap().name;
        if name.is_empty() {
            *name = format!("$arg{i}", i = self.index)
        }
        self.index += 1;
    }
}
