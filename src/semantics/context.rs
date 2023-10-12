use std::{
    collections::BTreeMap,
    sync::{Arc, Weak},
};

use crate::{
    ast::CallNamePart,
    compilation::Compiler,
    hir::{
        Expression, Function, FunctionDeclaration, FunctionNamePart, Module, Name,
        ParameterOrVariable, SelfType, TraitDeclaration, Type, TypeDeclaration, Typed,
        VariableDeclaration,
    },
    named::Named,
};

/// Trait for various AST lowering contexts
pub trait Context {
    /// Get compiler
    fn compiler(&self) -> &Compiler;

    /// Get compiler
    fn compiler_mut(&mut self) -> &mut Compiler;

    /// Get current module this context is for
    fn module(&self) -> &Module;

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module;

    /// Is this a context for builtin module?
    fn is_for_builtin_module(&self) -> bool {
        self.module().is_builtin
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext {
        let module = self.compiler().builtin_module().unwrap_or(self.module());
        debug_assert!(module.is_builtin);
        BuiltinContext { module }
    }

    /// Get current function
    fn function(&self) -> Option<Arc<FunctionDeclaration>>;

    /// Find type by name
    fn find_type(&self, name: &str) -> Option<Type>;

    /// Find variable by name
    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable>;

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>);

    /// Add trait to context
    fn add_trait(&mut self, tr: Arc<TraitDeclaration>);

    /// Add function to context
    fn add_function(&mut self, f: Function);

    /// Add variable to context
    fn add_variable(&mut self, v: Arc<VariableDeclaration>);

    /// Get all visible functions
    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function>;

    /// Get candidates for function call
    fn candidates(
        &self,
        name_parts: &[CallNamePart],
        args_cache: &[Option<Expression>],
    ) -> Vec<Function> {
        let mut functions = self.functions_with_n_name_parts(name_parts.len());
        // Add functions from traits
        functions.extend(
            args_cache
                .iter()
                .filter_map(|a| a.as_ref())
                .filter_map(|a| {
                    if let Type::Trait(tr) = a.ty() {
                        return Some(tr);
                    } else {
                        None
                    }
                })
                .flat_map(|tr| {
                    tr.functions_with_n_name_parts(name_parts.len())
                        .cloned()
                        .collect::<Vec<_>>()
                }),
        );

        // Filter functions by name parts
        functions
            .iter()
            .filter(|f| {
                f.name_parts().iter().zip(name_parts).enumerate().all(
                    |(i, (f_part, c_part))| match (f_part, c_part) {
                        (FunctionNamePart::Text(text1), CallNamePart::Text(text2)) => {
                            text1.as_str() == text2.as_str()
                        }
                        (FunctionNamePart::Parameter(_), CallNamePart::Argument(_)) => true,
                        (FunctionNamePart::Parameter(_), CallNamePart::Text(_)) => {
                            args_cache[i].is_some()
                        }
                        _ => false,
                    },
                )
            })
            .cloned()
            .collect()
    }

    /// Get function with same name
    fn function_with_name(&self, name: &str) -> Option<Function>;

    /// Get all functions with same name format
    fn functions_with_format(&self, format: &str) -> BTreeMap<Name, Function>;

    /// Find concrete function for trait function
    fn find_implementation(&self, trait_fn: &Function, self_type: &Type) -> Option<Function> {
        let funcs = self.functions_with_n_name_parts(trait_fn.name_parts().len());
        funcs
            .iter()
            .find(|f| {
                trait_fn
                    .name_parts()
                    .iter()
                    .zip(f.name_parts())
                    .all(|(a, b)| match (a, b) {
                        (FunctionNamePart::Text(a), FunctionNamePart::Text(b)) => {
                            a.as_str() == b.as_str()
                        }
                        (FunctionNamePart::Parameter(a), FunctionNamePart::Parameter(b)) => {
                            a.ty().map_self(self_type) == &b.ty()
                        }
                        _ => false,
                    })
                    && trait_fn.return_type().map_self(self_type) == &f.return_type()
            })
            .cloned()
    }
}

/// Context that is a child of another context
pub trait ChildContext {
    /// Get parent context
    fn parent(&self) -> &dyn Context;

    /// Get parent context
    fn parent_mut(&mut self) -> &mut dyn Context;

    /// Get compiler
    fn compiler(&self) -> &Compiler {
        self.parent().compiler()
    }

    /// Get compiler
    fn compiler_mut(&mut self) -> &mut Compiler {
        self.parent_mut().compiler_mut()
    }

    /// Get current module this context is for
    fn module(&self) -> &Module {
        self.parent().module()
    }

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module {
        self.parent_mut().module_mut()
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext {
        self.parent().builtin()
    }

    /// Get current function
    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        self.parent().function()
    }

    /// Find type by name without checking parent context
    fn find_type_here(&self, _name: &str) -> Option<Type> {
        None
    }

    /// Find type by name
    fn find_type(&self, name: &str) -> Option<Type> {
        self.find_type_here(name)
            .or_else(|| self.parent().find_type(name))
    }

    /// Find variable by name without checking parent context
    fn find_variable_here(&self, _name: &str) -> Option<ParameterOrVariable> {
        None
    }

    /// Find variable by name
    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        self.find_variable_here(name)
            .or_else(|| self.parent().find_variable(name))
    }

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        self.parent_mut().add_type(ty)
    }

    /// Add trait to context
    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.parent_mut().add_trait(tr)
    }

    /// Add function to context
    fn add_function(&mut self, f: Function) {
        self.parent_mut().add_function(f)
    }

    /// Add variable to context
    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        self.parent_mut().add_variable(v)
    }

    /// Get all visible functions without checking parent context
    fn functions_with_n_name_parts_here(&self, _n: usize) -> Vec<Function> {
        vec![]
    }

    /// Get all visible functions
    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        self.functions_with_n_name_parts_here(n)
            .into_iter()
            .chain(self.parent().functions_with_n_name_parts(n))
            .collect()
    }

    /// Get function with same name without checking parent context
    fn function_with_name_here(&self, _name: &str) -> Option<Function> {
        None
    }

    /// Get function with same name
    fn function_with_name(&self, name: &str) -> Option<Function> {
        self.function_with_name_here(name)
            .or_else(|| self.parent().function_with_name(name))
    }

    /// Get all functions with same name format
    fn functions_with_format_here(&self, _format: &str) -> BTreeMap<Name, Function> {
        BTreeMap::new()
    }

    /// Get all functions with same name format
    fn functions_with_format(&self, format: &str) -> BTreeMap<Name, Function> {
        self.functions_with_format_here(format)
            .into_iter()
            .chain(self.parent().functions_with_format(format).into_iter())
            .collect()
    }
}

impl<CC: ChildContext> Context for CC {
    fn compiler(&self) -> &Compiler {
        (self as &CC).compiler()
    }

    fn compiler_mut(&mut self) -> &mut Compiler {
        (self as &mut CC).compiler_mut()
    }

    /// Get current module this context is for
    fn module(&self) -> &Module {
        (self as &CC).module()
    }

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module {
        (self as &mut CC).module_mut()
    }

    /// Get current function
    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        (self as &CC).function()
    }

    /// Find type by name
    fn find_type(&self, name: &str) -> Option<Type> {
        (self as &CC).find_type(name)
    }

    /// Find variable by name
    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        (self as &CC).find_variable(name)
    }

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        (self as &mut CC).add_type(ty)
    }

    /// Add trait to context
    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        (self as &mut CC).add_trait(tr)
    }

    /// Add function to context
    fn add_function(&mut self, f: Function) {
        (self as &mut CC).add_function(f)
    }

    /// Add variable to context
    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        (self as &mut CC).add_variable(v)
    }

    /// Get all visible functions
    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        (self as &CC).functions_with_n_name_parts(n)
    }

    /// Get function with same name
    fn function_with_name(&self, name: &str) -> Option<Function> {
        (self as &CC).function_with_name(name)
    }

    /// Get all functions with same name format
    fn functions_with_format(&self, format: &str) -> BTreeMap<Name, Function> {
        (self as &CC).functions_with_format(format)
    }
}

/// Helper struct to get builtin things
pub struct BuiltinContext<'m> {
    /// Builtin module
    module: &'m Module,
}

impl<'m> BuiltinContext<'m> {
    /// Get builtin types
    pub fn types(&self) -> BuiltinTypes<'m> {
        BuiltinTypes {
            module: self.module,
        }
    }
}

/// Helper struct to get builtin types
pub struct BuiltinTypes<'m> {
    /// Builtin module
    module: &'m Module,
}

/// Helper macro to add builtin types
macro_rules! builtin_types {
    ($($name: ident),*) => {
        $(pub fn $name(&self) -> Type {
            let name = stringify!($name);
            self.get_type(&format!("{}{}", name[0..1].to_uppercase(), &name[1..]))
        })*
    };
}

impl BuiltinTypes<'_> {
    /// Get builtin type by name
    fn get_type(&self, name: &str) -> Type {
        self.module.types.get(name).unwrap().clone().into()
    }

    builtin_types!(none, bool, integer, rational, string);
}

/// Context for lowering content of module
pub struct ModuleContext<'c> {
    /// Module, which is being lowered
    pub module: Module,
    /// Compiler for modules
    pub compiler: &'c mut Compiler,
}

impl<'c> ModuleContext<'c> {
    pub fn new(compiler: &'c mut Compiler) -> Self {
        Self {
            module: Module::default(),
            compiler,
        }
    }
}

impl Context for ModuleContext<'_> {
    fn compiler(&self) -> &Compiler {
        self.compiler
    }

    fn compiler_mut(&mut self) -> &mut Compiler {
        self.compiler
    }

    fn module(&self) -> &Module {
        &self.module
    }

    fn module_mut(&mut self) -> &mut Module {
        &mut self.module
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        None
    }

    fn find_type(&self, name: &str) -> Option<Type> {
        let ty = self.module.types.get(name).cloned().map(|t| t.into());
        if ty.is_none() && !self.module.is_builtin {
            return Module::builtin().types.get(name).cloned().map(|t| t.into());
        }
        ty
    }

    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        let var = self.module.variables.get(name).cloned().map(|v| v.into());
        if var.is_none() && !self.module.is_builtin {
            return Module::builtin()
                .variables
                .get(name)
                .cloned()
                .map(|v| v.into());
        }
        var
    }

    /// Add type to context
    fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
        self.module.types.insert(ty.name().to_string(), ty.into());
    }

    fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
        self.module.types.insert(tr.name().to_string(), tr.into());
    }

    fn add_function(&mut self, f: Function) {
        self.module.insert_function(f);
    }

    fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
        self.module.variables.insert(v.name().to_string(), v);
    }

    fn function_with_name(&self, name: &str) -> Option<Function> {
        let f = self
            .module
            .functions
            .values()
            .find_map(|fs| fs.get(name).cloned());
        if f.is_none() && !self.module.is_builtin {
            return Module::builtin()
                .functions
                .values()
                .find_map(|fs| fs.get(name).cloned());
        }
        f
    }

    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        let mut functions: Vec<_> = self
            .module
            .functions_with_n_name_parts(n)
            .cloned()
            .collect();
        if !self.module.is_builtin {
            functions.extend(Module::builtin().functions_with_n_name_parts(n).cloned());
        }
        functions
    }

    fn functions_with_format(&self, format: &str) -> BTreeMap<Name, Function> {
        let mut funcs = self
            .module
            .functions
            .get(format)
            .cloned()
            .unwrap_or_default();
        if !self.module.is_builtin {
            funcs.extend(
                Module::builtin()
                    .functions
                    .get(format)
                    .cloned()
                    .unwrap_or_default(),
            )
        }
        funcs
    }
}

/// Context for lowering body of function
pub struct FunctionContext<'p> {
    /// Function, which is being lowered
    pub function: Arc<FunctionDeclaration>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl ChildContext for FunctionContext<'_> {
    fn parent(&self) -> &dyn Context {
        self.parent
    }

    fn parent_mut(&mut self) -> &mut dyn Context {
        self.parent
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        Some(self.function.clone())
    }

    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.function
            .parameters()
            .find(|p| p.name() == name)
            .map(|p| p.into())
    }

    fn add_type(&mut self, _ty: Arc<TypeDeclaration>) {
        todo!("local types")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("local traits")
    }

    fn add_function(&mut self, f: Function) {
        // TODO: local functions
        self.parent.add_function(f)
    }

    fn add_variable(&mut self, _v: Arc<VariableDeclaration>) {
        todo!("local variables")
    }
}

/// Context for lowering body of trait
pub struct TraitContext<'p> {
    /// Trait, which is being lowered
    pub tr: TraitDeclaration,

    /// Uninitialized weak pointer to trait
    pub trait_weak: Weak<TraitDeclaration>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl ChildContext for TraitContext<'_> {
    fn parent(&self) -> &dyn Context {
        self.parent
    }

    fn parent_mut(&mut self) -> &mut dyn Context {
        self.parent
    }

    fn find_type_here(&self, name: &str) -> Option<Type> {
        if name != "Self" {
            return None;
        }

        Some(
            SelfType {
                associated_trait: self.trait_weak.clone(),
            }
            .into(),
        )
    }

    fn add_type(&mut self, _ty: Arc<TypeDeclaration>) {
        todo!("types in traits")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("traits in traits?")
    }

    fn add_function(&mut self, f: Function) {
        self.tr.functions.push(f)
    }

    fn add_variable(&mut self, _v: Arc<VariableDeclaration>) {
        todo!("variables in traits")
    }

    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        self.tr
            .functions
            .iter()
            .filter(move |f| f.name_parts().len() == n)
            .cloned()
            .collect()
    }
}

/// Context for introducing generic parameters
pub struct GenericContext<'p> {
    /// Types of generic parameters
    pub generic_parameters: Vec<Type>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl ChildContext for GenericContext<'_> {
    fn parent(&self) -> &dyn Context {
        self.parent
    }

    fn parent_mut(&mut self) -> &mut dyn Context {
        self.parent
    }

    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.generic_parameters
            .iter()
            .find(|p| p.name() == name)
            .cloned()
    }
}
