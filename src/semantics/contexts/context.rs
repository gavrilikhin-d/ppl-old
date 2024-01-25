use std::fmt::Display;

use crate::{
    compilation::Compiler,
    hir::{Function, FunctionData, FunctionNamePart, Module, Type, Typed},
    semantics::{AddDeclaration, ConvertibleTo, FindDeclaration},
};

use super::BuiltinContext;

/// Trait for various AST lowering contexts
pub trait Context: FindDeclaration + AddDeclaration + Display {
    /// Get parent context
    fn parent(&self) -> Option<&dyn Context> {
        None
    }

    /// Get parent context
    fn parent_mut(&mut self) -> Option<&mut dyn Context> {
        None
    }

    /// Get compiler
    fn compiler(&self) -> &Compiler {
        Context::parent(self).unwrap().compiler()
    }

    /// Get compiler
    fn compiler_mut(&mut self) -> &mut Compiler {
        Context::parent_mut(self).unwrap().compiler_mut()
    }

    /// Get current module this context is for
    fn module(&self) -> &Module {
        Context::parent(self).unwrap().module()
    }

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module {
        Context::parent_mut(self).unwrap().module_mut()
    }

    /// Get current function
    fn function(&self) -> Option<Function> {
        Context::parent(self).and_then(|p| p.function())
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext {
        let module = self.compiler().builtin_module().unwrap_or(self.module());
        BuiltinContext { module }
    }

    /// Find concrete function for trait function
    fn find_implementation(&mut self, trait_fn: &FunctionData, self_type: &Type) -> Option<Function>
    where
        Self: Sized,
    {
        let funcs = self.functions_with_n_name_parts(trait_fn.name_parts().len());
        funcs
            .iter()
            .find(|f| {
                trait_fn
                    .name_parts()
                    .iter()
                    .zip(f.read().unwrap().name_parts())
                    .all(|(a, b)| match (a, b) {
                        (FunctionNamePart::Text(a), FunctionNamePart::Text(b)) => {
                            a.as_str() == b.as_str()
                        }
                        (FunctionNamePart::Parameter(a), FunctionNamePart::Parameter(b)) => a
                            .ty()
                            .map_self(self_type)
                            .clone()
                            .convertible_to(b.ty())
                            .within(self)
                            .is_ok_and(|converible| converible),
                        _ => false,
                    })
                    && trait_fn
                        .return_type
                        .map_self(self_type)
                        .clone()
                        // TODO: real return type range
                        .convertible_to(f.read().unwrap().return_type.clone())
                        .within(self)
                        .is_ok_and(|convertible| convertible)
            })
            .cloned()
    }

    /// Debug function to print hierarchy of contexts
    fn print_contexts_hierarchy(&self)
    where
        Self: Sized,
    {
        println!("Contexts hierarchy:");

        let mut i = 0;
        let mut current = self as &dyn Context;
        loop {
            println!("{i}) {current}");
            if let Some(parent) = Context::parent(current) {
                current = parent;
                i += 1;
            } else {
                return;
            }
        }
    }
}
