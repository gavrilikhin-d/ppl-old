use std::collections::BTreeMap;

use crate::{
    ast::CallNamePart,
    hir::{Expression, Function, FunctionNamePart, Module, Name, ParameterOrVariable, Type, Typed},
};

/// Trait to find declaration at current level
pub trait FindDeclarationHere {
    /// Find type by name without checking parent context
    fn find_type_here(&self, name: &str) -> Option<Type> {
        let _ = name;
        None
    }

    /// Find variable by name without checking parent context
    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        let _ = name;
        None
    }

    /// Get all visible functions without checking parent context
    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        let _ = n;
        vec![]
    }

    /// Get function with same name without checking parent context
    fn function_with_name_here(&self, name: &str) -> Option<Function> {
        let _ = name;
        None
    }

    /// Get all functions with same name format
    fn functions_with_format_here(&self, format: &str) -> BTreeMap<Name, Function> {
        let _ = format;
        BTreeMap::new()
    }
}

/// Trait to find declaration at current level or above
pub trait FindDeclaration: FindDeclarationHere {
    /// Get parent to find declaration in
    fn parent(&self) -> Option<&dyn FindDeclaration> {
        None
    }

    /// Find type by name
    fn find_type(&self, name: &str) -> Option<Type> {
        self.find_type_here(name)
            .or_else(|| self.parent().and_then(|p| p.find_type(name)))
    }

    /// Find variable by name
    fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
        self.find_variable_here(name)
            .or_else(|| self.parent().and_then(|p| p.find_variable(name)))
    }

    /// Get all visible functions
    fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
        self.functions_with_n_name_parts_here(n)
            .into_iter()
            .chain(
                self.parent()
                    .and_then(|p| Some(p.functions_with_n_name_parts(n)))
                    .unwrap_or_default(),
            )
            .collect()
    }

    /// Get function with same name
    fn function_with_name(&self, name: &str) -> Option<Function> {
        self.function_with_name_here(name)
            .or_else(|| self.parent().and_then(|p| p.function_with_name(name)))
    }

    /// Get all functions with same name format
    fn functions_with_format(&self, format: &str) -> BTreeMap<Name, Function> {
        self.functions_with_format_here(format)
            .into_iter()
            .chain(
                self.parent()
                    .and_then(|p| Some(p.functions_with_format(format)))
                    .unwrap_or_default()
                    .into_iter(),
            )
            .collect()
    }

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

impl FindDeclarationHere for Module {
    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.types.get(name).cloned().map(|t| t.into())
    }

    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.variables.get(name).cloned().map(|v| v.into())
    }

    fn function_with_name_here(&self, name: &str) -> Option<Function> {
        self.functions.values().find_map(|fs| fs.get(name).cloned())
    }

    fn functions_with_format_here(&self, format: &str) -> BTreeMap<Name, Function> {
        self.functions.get(format).cloned().unwrap_or_default()
    }

    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        self.functions_with_n_name_parts(n).cloned().collect()
    }
}

impl FindDeclaration for Module {}

impl<M: AsRef<Module>> FindDeclarationHere for M {
    fn find_type_here(&self, name: &str) -> Option<Type> {
        self.as_ref().find_type_here(name)
    }

    fn find_variable_here(&self, name: &str) -> Option<ParameterOrVariable> {
        self.as_ref().find_variable_here(name)
    }

    fn function_with_name_here(&self, name: &str) -> Option<Function> {
        self.as_ref().function_with_name_here(name)
    }

    fn functions_with_format_here(&self, format: &str) -> BTreeMap<Name, Function> {
        self.as_ref().functions_with_format_here(format)
    }

    fn functions_with_n_name_parts_here(&self, n: usize) -> Vec<Function> {
        self.as_ref().functions_with_n_name_parts_here(n)
    }
}
