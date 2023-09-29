use std::{collections::HashMap, path::Path, sync::Arc};

use inkwell::{context::Context, execution_engine::ExecutionEngine, OptimizationLevel};

use crate::hir::Module;

pub struct Compiler<'llvm> {
    pub llvm: &'llvm Context,
    pub engine: ExecutionEngine<'llvm>,
    pub modules: HashMap<String, Arc<Module>>,
}

impl<'llvm> Compiler<'llvm> {
    /* TODO: settings (Optimization, etc) */
    pub fn new(llvm: &'llvm Context) -> Self {
        let engine = llvm
            .create_module("")
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        // TODO: env var for runtime path
        let runtime_folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime");
        let lib_path = runtime_folder
            .join("target/debug/libruntime.dylib")
            .to_str()
            .unwrap()
            .to_string();
        let error = inkwell::support::load_library_permanently(&lib_path);
        assert!(!error, "Failed to load runtime library at: {}", &lib_path);

        Self {
            llvm,
            engine,
            modules: HashMap::new(),
        }
    }
}
