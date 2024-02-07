use inkwell::{debug_info::{DICompileUnit, DebugInfoBuilder}, module::Module};

/// Builder for debug information
pub struct DebugInfo<'llvm> {
    /// Builder for debug info
    dibuilder: DebugInfoBuilder<'llvm>,
    /// Compile unit for debug info
    compile_unit: DICompileUnit<'llvm>,
}

impl<'llvm> DebugInfo<'llvm> {
    /// Create new debug info for module
    pub fn new(module: &Module<'llvm>) -> Self {
        let llvm = module.get_context();
        let debug_metadata_version = llvm.i32_type().const_int(5, false);
        module.add_basic_value_flag(
            "Debug Info Version",
            inkwell::module::FlagBehavior::Warning,
            debug_metadata_version,
        );
        let (dibuilder, compile_unit) = module.create_debug_info_builder(
            true,
            /* language */ inkwell::debug_info::DWARFSourceLanguage::C,
            /* filename */ module.get_source_file_name().to_str().unwrap(),
            /* directory */ ".",
            /* producer */ "ppl",
            /* is_optimized */ false,
            /* compiler command line flags */ "",
            /* runtime_ver */ 0,
            /* split_name */ "",
            /* kind */ inkwell::debug_info::DWARFEmissionKind::Full,
            /* dwo_id */ 0,
            /* split_debug_inling */ false,
            /* debug_info_for_profiling */ false,
            /* sys_root */ "/",
            /* sdk */ "",
        );

        Self {
            dibuilder,
            compile_unit,
        }
    }

    /// Finalize debug info
    pub fn finalize(&self) {
        self.dibuilder.finalize();
    }
}