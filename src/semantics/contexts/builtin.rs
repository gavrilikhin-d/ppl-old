use crate::hir::{Module, Type};

/// Helper struct to get builtin things
pub struct BuiltinContext<'m> {
    /// Builtin module
    pub module: &'m Module,
}

impl<'m> BuiltinContext<'m> {
    /// Get builtin types
    pub fn types(&self) -> BuiltinTypes<'m> {
        BuiltinTypes {
            module: self.module,
        }
    }
}

impl AsRef<Module> for BuiltinContext<'_> {
    fn as_ref(&self) -> &Module {
        self.module
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
