use crate::hir::{Module, Specialize, SpecializeClass, Type};

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
        self.module
            .types
            .get(name)
            .expect(format!("Builtin type `{name}` should be present").as_str())
            .clone()
            .into()
    }

    builtin_types!(none, bool, integer, rational, string);

    /// Get builtin type for types
    pub fn type_(&self) -> Type {
        self.get_type("Type")
    }

    /// Get `Type<T>` of this type
    pub fn type_of(&self, ty: Type) -> Type {
        self.type_()
            .specialize_with(
                self.type_()
                    .as_class()
                    .specialize_with(SpecializeClass::without_members(vec![ty]))
                    .into(),
            )
            .into()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        compilation::Compiler,
        hir::{Specialize, SpecializeClass, SpecializedType},
    };

    use super::BuiltinTypes;

    #[test]
    fn type_of() {
        let compiler = Compiler::new();
        let builtin = BuiltinTypes {
            module: compiler.builtin_module().unwrap(),
        };
        let none = builtin.none();
        let none_ty = builtin.type_of(none.clone());
        assert_eq!(
            none_ty,
            SpecializedType {
                generic: builtin.type_(),
                specialized: builtin
                    .type_()
                    .as_class()
                    .specialize_with(SpecializeClass::without_members(vec![none]))
                    .into()
            }
            .into()
        );
    }
}
