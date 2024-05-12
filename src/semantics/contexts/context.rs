use std::fmt::Display;

use crate::{
    compilation::Compiler,
    hir::{Function, FunctionData, FunctionNamePart, ModuleData, SelfType, Type, Typed},
    named::Named,
    semantics::{AddDeclaration, ConvertibleTo, FindDeclaration, Implements},
};

use super::{BuiltinContext, GenericContext};

use crate::DataHolder;

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
    fn module(&self) -> &ModuleData {
        Context::parent(self).unwrap().module()
    }

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut ModuleData {
        Context::parent_mut(self).unwrap().module_mut()
    }

    /// Get current function
    fn function(&self) -> Option<Function> {
        Context::parent(self).and_then(|p| p.function())
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext
    where
        Self: Sized,
    {
        BuiltinContext { context: self }
    }

    /// Find concrete function for trait function
    fn find_implementation(
        &mut self,
        trait_fn: &FunctionData,
        self_type_specialization: Option<Type>,
    ) -> Option<Function>
    where
        Self: Sized,
    {
        let self_ty: Type = SelfType {
            associated_trait: trait_fn.tr.clone().unwrap(),
        }
        .into();
        let mut context = GenericContext::for_generics(vec![self_ty.clone()], self);
        if let Some(concrete) = self_type_specialization.clone() {
            context.map_generic(self_ty, concrete);
        }
        let funcs = context.functions_with_n_name_parts(trait_fn.name_parts().len());
        funcs.into_iter().find(|f| {
            let params_ok = trait_fn
                .name_parts()
                .iter()
                .zip(f.read().unwrap().name_parts())
                .all(|(a, b)| match (a, b) {
                    (FunctionNamePart::Text(a), FunctionNamePart::Text(b)) => {
                        a.as_str() == b.as_str()
                    }
                    (FunctionNamePart::Parameter(a), FunctionNamePart::Parameter(b)) => a
                        .ty()
                        .convertible_to(b.ty())
                        .within(&mut context)
                        .is_ok_and(|converible| converible),
                    _ => false,
                });
            let ret_ok = trait_fn
                .return_type
                // TODO: real return type range
                .convertible_to(f.read().unwrap().return_type.clone())
                .within(&mut context)
                .is_ok_and(|convertible| convertible);

            params_ok && ret_ok
        })
    }

    /// Find destructor for type
    fn destructor_for(&mut self, ty: Type) -> Option<Function>
    where
        Self: Sized,
    {
        let ty = self.builtin().types().reference_mut_to(ty);
        let name = format!("destroy <:{ty}>");
        self.function_with_name(&name)
    }

    /// Find clone function for type
    fn clone_for(&mut self, ty: Type) -> Option<Function>
    where
        Self: Sized,
    {
        match ty {
            Type::Class(c) => c
                .implements(self.builtin().traits().clonnable())
                .within(self)
                .ok()?
                .into_iter()
                .next(),
            _ => None,
        }
    }

    /// Debug function to print hierarchy of contexts
    fn print_contexts_hierarchy(&self)
    where
        Self: Sized,
    {
        eprintln!("Contexts hierarchy:");

        let mut i = 0;
        let mut current = self as &dyn Context;
        loop {
            eprintln!("{i}) {current}");
            if let Some(parent) = Context::parent(current) {
                current = parent;
                i += 1;
            } else {
                return;
            }
        }
    }
}
