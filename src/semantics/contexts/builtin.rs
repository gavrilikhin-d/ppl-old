use std::fmt::Display;

use crate::hir::{SpecializeParameters, Type};

use super::Context;

/// Helper struct to get builtin things
pub struct BuiltinContext<'ctx> {
    /// Context to use lookup
    pub context: &'ctx dyn Context,
}

impl Display for BuiltinContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BuiltinContext")
    }
}

impl<'ctx> BuiltinContext<'ctx> {
    /// Get builtin types
    pub fn types(&self) -> BuiltinTypes<'ctx> {
        BuiltinTypes {
            context: self.context,
        }
    }
}

/// Helper struct to get builtin types
pub struct BuiltinTypes<'ctx> {
    /// Builtin module
    context: &'ctx dyn Context,
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
        self.context
            .find_type(name)
            .expect(format!("Builtin type `{name}` should be present",).as_str())
    }

    builtin_types!(none, bool, integer, rational, string, reference, i32, f64);

    /// Get builtin type for types
    pub fn type_(&self) -> Type {
        self.get_type("Type")
    }

    /// Get `Type<T>` of this type
    pub fn type_of(&self, ty: Type) -> Type {
        self.type_()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }

    /// Get `Reference<T>` for this type
    pub fn reference_to(&self, ty: Type) -> Type {
        self.reference()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }

    /// Get builtin type for `ReferenceMut<T>`
    pub fn reference_mut(&self) -> Type {
        self.get_type("ReferenceMut")
    }

    /// Get `ReferenceMut<T>` for this type
    pub fn reference_mut_to(&self, ty: Type) -> Type {
        self.reference_mut()
            .as_class()
            .specialize_parameters(std::iter::once(ty))
            .into()
    }
}
