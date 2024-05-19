use inkwell::{module::Module, types::FunctionType, values::FunctionValue};

use crate::ir::Types;

/// LLVM IR for PPL's functions
pub struct Functions<'llvm, 'm> {
    module: &'m Module<'llvm>,
}

// Macro to add builtin function
macro_rules! add_builtin_function {
    ($name:ident : ( $($args:ident),* ) -> $ret:ident ) => {
        pub fn $name(&self) -> FunctionValue<'llvm> {
            let types = Types::new(self.module.get_context());
            self.get_or_add_function(
                stringify!($name),
                types.$ret().fn_type(&[$(types.$args().into()),*], false),
            )
        }
    };
}

impl<'llvm, 'm> Functions<'llvm, 'm> {
    /// Initialize LLVM IR for PPL's functions
    pub fn new(module: &'m Module<'llvm>) -> Self {
        Self { module }
    }

    /// Get function by name
    pub fn get(&self, name: &str) -> Option<FunctionValue<'llvm>> {
        self.module.get_function(name)
    }

    /// Get function by name if it exists, or add a declaration for it
    pub fn get_or_add_function(&self, name: &str, ty: FunctionType<'llvm>) -> FunctionValue<'llvm> {
        if let Some(f) = self.module.get_function(&name) {
            return f;
        }
        self.module.add_function(name, ty, None)
    }

    // LLVM IR for constructor of [`Integer`](Type::Integer) type from i32
    add_builtin_function!(integer_from_i32: (i32) -> integer);

    // LLVM IR for constructor of [`Integer`](Type::Integer) type from i64
    add_builtin_function!(integer_from_i64: (i64) -> integer);

    // LLVM IR for constructor of [`Integer`](Type::Integer) type from C string
    add_builtin_function!(integer_from_c_string: (c_string) -> integer);

    // LLVM IR for constructor of `Rational` type from C string
    add_builtin_function!(rational_from_c_string: (c_string) -> rational);

    // LLVM IR for constructor of [`String`](Type::String) type from C string
    // and its length
    add_builtin_function!(
        string_from_c_string_and_length: (c_string, u64) -> string
    );

    // Create arc for class
    add_builtin_function!(
        create_arc: (u64) -> pointer
    );
}
