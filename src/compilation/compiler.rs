use std::{collections::HashMap, path::Path, sync::Arc};

use inkwell::{context::Context, execution_engine::ExecutionEngine, OptimizationLevel};
use log::debug;

use crate::{hir::Module, ir};

pub struct Compiler<'llvm> {
    pub llvm: &'llvm Context,
    pub engine: ExecutionEngine<'llvm>,
    pub modules: HashMap<String, Arc<Module>>,
}

impl<'llvm> Compiler<'llvm> {
    /* TODO: settings (Optimization, etc) */
    pub fn new(llvm: &'llvm Context) -> Self {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime/ppl.bc"));

        let builtin = inkwell::module::Module::parse_bitcode_from_path(path, llvm)
            .expect("Bytecode for builtin module not found or invalid");
        debug!(target: "builtin", "{}", builtin.to_string());

        let engine = builtin
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let functions = ir::Functions::new(&builtin);

        /// Macro to add global mapping
        macro_rules! add_global_mapping {
            ($name:ident) => {
                engine.add_global_mapping(&functions.$name(), runtime::$name as usize);
            };
        }

        add_global_mapping!(integer_from_i64);
        add_global_mapping!(integer_from_c_string);
        add_global_mapping!(rational_from_c_string);
        add_global_mapping!(string_from_c_string_and_length);
        add_global_mapping!(integer_as_string);
        add_global_mapping!(print_string);
        add_global_mapping!(minus_integer);
        add_global_mapping!(integer_plus_integer);
        add_global_mapping!(integer_star_integer);
        add_global_mapping!(integer_eq_integer);
        add_global_mapping!(integer_less_integer);

        Self {
            llvm,
            engine,
            modules: HashMap::new(),
        }
    }
}
