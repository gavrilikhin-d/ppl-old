use std::fmt::Display;

use crate::{
    hir::{Class, Function, ParameterOrVariable, Trait, Type, Variable},
    named::Named,
    semantics::{AddDeclaration, FindDeclaration, FindDeclarationHere},
    DataHolder,
};

use super::Context;

/// Context for lowering body of function
pub struct FunctionContext<'p> {
    /// Function, which is being lowered
    pub function: Function,

    /// Local variables declared so far
    pub variables: Vec<Variable>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl Display for FunctionContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "FunctionContext")?;
        writeln!(f, "\tfor function: {}", self.function.name())
    }
}

impl FindDeclarationHere for FunctionContext<'_> {
    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.variables
            .iter()
            .cloned()
            .find(|p| p.name() == name)
            .map(|p| p.into())
            .or_else(|| {
                self.function
                    .read()
                    .unwrap()
                    .parameters()
                    .find(|p| p.name() == name)
                    .map(|p| p.into())
            })
    }

    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.function
            .read()
            .unwrap()
            .generic_types
            .iter()
            .find(|p| p.name() == name)
            .cloned()
    }
}

impl FindDeclaration for FunctionContext<'_> {
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        Some(self.parent as _)
    }

    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        let mut functions: Vec<_> = self
            .functions_with_n_name_parts_here(n)
            .into_iter()
            .chain(
                FindDeclaration::parent(self)
                    .and_then(|p| Some(p.functions_with_n_name_parts(n)))
                    .unwrap_or_default(),
            )
            .collect();
        // TODO: allow recursion, if has @recursive
        functions.retain(|f| *f != self.function);
        functions
    }
}

impl AddDeclaration for FunctionContext<'_> {
    fn parent_mut(&mut self) -> Option<&mut dyn AddDeclaration> {
        Some(self.parent)
    }

    fn add_type(&mut self, _ty: Class) {
        todo!("local types")
    }

    fn add_trait(&mut self, _tr: Trait) {
        todo!("local traits")
    }

    fn add_function(&mut self, f: Function) {
        // TODO: local functions
        self.parent.add_function(f)
    }

    fn add_variable(&mut self, v: Variable) {
        self.variables.push(v)
    }
}

impl Context for FunctionContext<'_> {
    fn parent(&self) -> Option<&dyn Context> {
        Some(self.parent)
    }

    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        Some(self.parent)
    }

    fn function(&self) -> Option<Function> {
        Some(self.function.clone())
    }
}
