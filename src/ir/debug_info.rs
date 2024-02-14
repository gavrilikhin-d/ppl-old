use inkwell::{
    context::ContextRef,
    debug_info::{
        AsDIScope, DIBasicType, DICompileUnit, DIFile, DIFlagsConstants, DILocation, DIScope,
        DISubprogram, DISubroutineType, DIType, DebugInfoBuilder,
    },
    module::Module,
    values::FunctionValue,
};

use crate::{ColumnNumber, LineNumber};

/// Builder for debug information
pub struct DebugInfo<'llvm> {
    /// LLVM context
    llvm: ContextRef<'llvm>,
    /// Builder for debug info
    dibuilder: DebugInfoBuilder<'llvm>,
    /// Compile unit for debug info
    compile_unit: DICompileUnit<'llvm>,
    /// Scopes stack
    scopes: Vec<DIScope<'llvm>>,
}

impl<'llvm> DebugInfo<'llvm> {
    /// Create new debug info for module
    pub fn new(module: &Module<'llvm>) -> Self {
        let llvm = module.get_context();
        let debug_metadata_version = llvm.i32_type().const_int(3, false);
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
            llvm,
            dibuilder,
            compile_unit,
            scopes: vec![compile_unit.get_file().as_debug_info_scope()],
        }
    }

    /// Finalize debug info
    pub fn finalize(&self) {
        self.dibuilder.finalize();
    }

    /// Get current file
    pub fn file(&self) -> DIFile<'llvm> {
        self.compile_unit.get_file()
    }

    /// Get current scope
    pub fn scope(&self) -> DIScope<'llvm> {
        self.scopes.last().unwrap().clone()
    }

    /// Get debug info for i32 type
    pub fn i32(&self) -> DIBasicType<'llvm> {
        let size_in_bits = 32;
        let encoding = gimli::DW_ATE_signed.0 as u32;
        let flags = DIFlagsConstants::ZERO;
        self.dibuilder
            .create_basic_type("i32", size_in_bits, encoding, flags)
            .unwrap()
    }

    /// Get debug info for function type
    pub fn fn_type(&self, ret: DIType<'llvm>, args: &[DIType<'llvm>]) -> DISubroutineType<'llvm> {
        self.dibuilder
            .create_subroutine_type(self.file(), Some(ret), args, DIFlagsConstants::ZERO)
    }

    /// Get debug location
    pub fn location(&self, line: LineNumber, column: ColumnNumber) -> DILocation<'llvm> {
        let line = line.zero_based() as u32;
        let column = column.zero_based() as u32;
        let inlined_at = None;
        self.dibuilder
            .create_debug_location(self.llvm, line, column, self.scope(), inlined_at)
    }

    /// Register function in debug info
    pub fn register_function(
        &self,
        f: FunctionValue<'llvm>,
        at: LineNumber,
    ) -> DISubprogram<'llvm> {
        let name = f.get_name().to_str().unwrap();
        let linkage_name = None;
        let line_no = at.zero_based() as u32;
        let ditype = self.fn_type(self.i32().as_type(), &[]);
        let is_local_to_unit = false;
        let is_definition = f.count_basic_blocks() > 0;
        let scope_line = 0;
        let flags = DIFlagsConstants::ZERO;
        let is_optimized = false;

        let subprogram = self.dibuilder.create_function(
            self.scope(),
            name,
            linkage_name,
            self.file(),
            line_no,
            ditype,
            is_local_to_unit,
            is_definition,
            scope_line,
            flags,
            is_optimized,
        );

        f.set_subprogram(subprogram);

        subprogram
    }
}
