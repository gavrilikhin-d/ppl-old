use inkwell::{module::Module, values::FunctionValue, types::FunctionType};

use crate::ir::Types;

/// LLVM IR for PPL's functions
pub struct Functions<'llvm, 'm> {
    module: &'m Module<'llvm>,
}

// Macro to add builtin function
macro_rules! add_builtin_function {
	($name:ident, $ret:ident, $($args:ident),*) => {
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

    // LLVM IR for default constructor of [`None`](Type::None) type
	add_builtin_function!(none, none,);

    /// LLVM IR for constructor of [`Integer`](Type::Integer) type from i64
    pub fn integer_from_i64(&self) -> FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_from_i64",
            types.integer().fn_type(&[types.i(64).into()], false),
        )
    }

    // LLVM IR for constructor of [`Integer`](Type::Integer) type from C string
	add_builtin_function!(integer_from_c_string, integer, c_string);

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

    // LLVM IR for "<:Integer> as String -> String" builtin function
	add_builtin_function!(integer_as_string, string, integer);

    // LLVM IR for "print <str: String> -> None" builtin function
	add_builtin_function!(print_string, none, string);

    // LLVM IR for "- <:Integer> -> Integer" builtin function
	add_builtin_function!(minus_integer, integer, integer);

	// LLVM IR for "<:Integer> + <:Integer> -> Integer" builtin function
   	add_builtin_function!(integer_plus_integer, integer, integer, integer);

   	// LLVM IR for "<:Integer> * <:Integer> -> Integer" builtin function
	add_builtin_function!(integer_star_integer, integer, integer, integer);

	// LLVM IR for "<:Integer> == <:Integer> -> Bool" builtin function
	add_builtin_function!(integer_eq_integer, bool, integer, integer);

	// LLVM IR for "<:Integer> < <:Integer> -> Bool" builtin function
	add_builtin_function!(integer_less_integer, bool, integer, integer);

    // IMPORTANT: don't forget to update global mapping when adding new function!!!
}