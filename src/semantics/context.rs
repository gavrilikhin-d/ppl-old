use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Weak},
};

use crate::{
    ast::CallNamePart,
    hir::{
        Expression, Function, FunctionDeclaration, FunctionNamePart, GenericType, Module, Name,
        ParameterOrVariable, SelfType, TraitDeclaration, Type, TypeDeclaration, Typed,
        VariableDeclaration,
    },
    named::Named,
};

/// Trait for various AST lowering contexts
pub trait Context {
    /// Get current module this context is for
    fn module(&self) -> &Module;

    /// Get current module this context is for
    fn module_mut(&mut self) -> &mut Module;

    /// Is this a context for builtin module?
    fn is_for_builtin_module(&self) -> bool {
        self.module().is_builtin
    }

    /// Get module context of builtin module
    fn builtin(&self) -> BuiltinContext;

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
    fn functions_with_format(&self, format: &str) -> HashMap<Name, Function>;

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

    /// Resolve module by name.
    ///
    /// # Algorithm
    /// In the directory of current module
    /// 1. Look for `{name}.ppl`
    /// 2. Look for `{name}/mod.ppl`
    fn resolve_module(&mut self, name: &str) -> miette::Result<Module>;
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

impl BuiltinTypes<'_> {
    /// Get builtin type by name
    fn get_type(&self, name: &str) -> Type {
        self.module
            .types
            .get(name)
            .expect(format!("Bultin types should have {name}").as_str())
            .clone()
            .into()
    }

    /// Get builtin "None" type
    pub fn none(&self) -> Type {
        self.get_type("None")
    }

    /// Get builtin "Bool" type
    pub fn bool(&self) -> Type {
        self.get_type("Bool")
    }

    /// Get builtin "Integer" type
    pub fn integer(&self) -> Type {
        self.get_type("Integer")
    }

    /// Get builtin "Rational" type
    pub fn rational(&self) -> Type {
        self.get_type("Rational")
    }

    /// Get builtin "String" type
    pub fn string(&self) -> Type {
        self.get_type("String")
    }
}

/// Context for lowering content of module
pub struct ModuleContext {
    /// Module, which is being lowered
    pub module: Module,
}

impl Default for ModuleContext {
    fn default() -> Self {
        Self {
            module: Module::default(),
        }
    }
}

impl Context for ModuleContext {
    fn module(&self) -> &Module {
        &self.module
    }

    fn module_mut(&mut self) -> &mut Module {
        &mut self.module
    }

    fn builtin(&self) -> BuiltinContext {
        if self.module.is_builtin {
            BuiltinContext {
                module: &self.module,
            }
        } else {
            BuiltinContext {
                module: Module::builtin(),
            }
        }
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

    fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
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

    /// Resolve module by name.
    ///
    /// # Algorithm
    /// In the directory of current module
    /// 1. Look for `{name}.ppl`
    /// 2. Look for `{name}/mod.ppl`
    fn resolve_module(&mut self, name: &str) -> miette::Result<Module> {
        let dir = Path::new(&self.module.filename)
            .parent()
            .unwrap_or(Path::new(""));

        // TODO: caching
        // TODO: better errors
        let mut path = dir.join(format!("{}.ppl", name));
        if !path.exists() {
            path = dir.join(name).join("mod.ppl");
        }
        Module::from_file_with_builtin(&path, self.module.is_builtin)
    }
}

/// Context for lowering body of function
pub struct FunctionContext<'p> {
    /// Function, which is being lowered
    pub function: Arc<FunctionDeclaration>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl Context for FunctionContext<'_> {
    fn module(&self) -> &Module {
        &self.parent.module()
    }

    fn module_mut(&mut self) -> &mut Module {
        self.parent.module_mut()
    }

    fn builtin(&self) -> BuiltinContext {
        self.parent.builtin()
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        Some(self.function.clone())
    }

    fn find_type(&self, name: &str) -> Option<Type> {
        self.parent.find_type(name)
    }

    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        self.function
            .parameters()
            .find(|p| p.name() == name)
            .map(|p| p.into())
            .or_else(|| self.parent.find_variable(name))
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

    fn function_with_name(&self, name: &str) -> Option<Function> {
        self.parent.function_with_name(name)
    }

    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        self.parent.functions_with_n_name_parts(n)
    }

    fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
        self.parent.functions_with_format(format)
    }

    fn resolve_module(&mut self, name: &str) -> miette::Result<Module> {
        self.parent.resolve_module(name)
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

impl Context for TraitContext<'_> {
    fn module(&self) -> &Module {
        &self.parent.module()
    }

    fn module_mut(&mut self) -> &mut Module {
        self.parent.module_mut()
    }

    fn builtin(&self) -> BuiltinContext {
        self.parent.builtin()
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        self.parent.function()
    }

    fn find_type(&self, name: &str) -> Option<Type> {
        if name == "Self" {
            return Some(
                SelfType {
                    associated_trait: self.trait_weak.clone(),
                }
                .into(),
            );
        }
        self.parent.find_type(name)
    }

    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        self.parent.find_variable(name)
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

    fn function_with_name(&self, name: &str) -> Option<Function> {
        self.parent.function_with_name(name)
    }

    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        let mut functions = self.parent.functions_with_n_name_parts(n);
        functions.extend(
            self.tr
                .functions
                .iter()
                .filter(move |f| f.name_parts().len() == n)
                .cloned(),
        );
        functions
    }

    fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
        self.parent.functions_with_format(format)
    }

    fn resolve_module(&mut self, name: &str) -> miette::Result<Module> {
        self.parent.resolve_module(name)
    }
}

/// Context for lowering body of types
pub struct TypeContext<'p> {
    /// Types of generic parameters
    pub generic_parameters: Vec<GenericType>,

    /// Parent context for this function
    pub parent: &'p mut dyn Context,
}

impl Context for TypeContext<'_> {
    fn module(&self) -> &Module {
        &self.parent.module()
    }

    fn module_mut(&mut self) -> &mut Module {
        self.parent.module_mut()
    }

    fn builtin(&self) -> BuiltinContext {
        self.parent.builtin()
    }

    fn function(&self) -> Option<Arc<FunctionDeclaration>> {
        self.parent.function()
    }

    fn find_type(&self, name: &str) -> Option<Type> {
        self.generic_parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| Type::Generic(p.clone()))
            .or_else(|| self.parent.find_type(name))
    }

    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        self.parent.find_variable(name)
    }

    fn add_type(&mut self, _ty: Arc<TypeDeclaration>) {
        todo!("types inside types")
    }

    fn add_trait(&mut self, _tr: Arc<TraitDeclaration>) {
        todo!("traits inside types")
    }

    fn add_function(&mut self, _f: Function) {
        unreachable!("functions inside types")
    }

    fn add_variable(&mut self, _v: Arc<VariableDeclaration>) {
        unreachable!("variables inside types")
    }

    fn function_with_name(&self, name: &str) -> Option<Function> {
        self.parent.function_with_name(name)
    }

    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        self.parent.functions_with_n_name_parts(n)
    }

    fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
        self.parent.functions_with_format(format)
    }

    fn resolve_module(&mut self, name: &str) -> miette::Result<Module> {
        self.parent.resolve_module(name)
    }
}
