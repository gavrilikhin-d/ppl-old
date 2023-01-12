use inkwell::{module::Module, values::FunctionValue, types::FunctionType};

use crate::ir::Types;

/// LLVM IR for PPL's functions
pub struct Functions<'llvm, 'm> {
    module: &'m Module<'llvm>,
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
    pub fn get_or_add_function(
        &self,
        name: &str,
        ty: FunctionType<'llvm>,
    ) -> FunctionValue<'llvm> {
        if let Some(f) = self.module.get_function(&name) {
            return f;
        }
        self.module.add_function(name, ty, None)
    }

    /// LLVM IR for default constructor of [`None`](Type::None) type
    pub fn none(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function("none", types.none().fn_type(&[], false))
    }

    /// LLVM IR for constructor of [`Integer`](Type::Integer) type from i64
    pub fn integer_from_i64(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_from_i64",
            types.integer().fn_type(&[types.i(64).into()], false),
        )
    }

    /// LLVM IR for constructor of [`Integer`](Type::Integer) type from C string
    pub fn integer_from_c_string(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_from_c_string",
            types.integer().fn_type(&[types.c_string().into()], false),
        )
    }

    /// LLVM IR for constructor of [`String`](Type::String) type from C string
    /// and its length
    pub fn string_from_c_string_and_length(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "string_from_c_string_and_length",
            types
                .string()
                .fn_type(&[types.c_string().into(), types.u(64).into()], false),
        )
    }

    /// LLVM IR for "<:Integer> as String -> String" builtin function
    pub fn integer_as_string(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_as_string",
            types.string().fn_type(&[types.integer().into()], false),
        )
    }

    /// LLVM IR for "print <str: String> -> None" builtin function
    pub fn print_string(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "print_string",
            types.none().fn_type(&[types.string().into()], false),
        )
    }

    /// LLVM IR for "- <:Integer> -> Integer" builtin function
    pub fn minus_integer(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "minus_integer",
            types.integer().fn_type(&[types.integer().into()], false),
        )
    }

    // IMPORTANT: don't forget to update global mapping when adding new function!!!
}