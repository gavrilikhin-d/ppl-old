use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;

use crate::{
    hir::{ParameterOrVariable, Statement},
    named::Named,
};

use super::{Functions, LocalHIRLowering, Types};

/// Trait for common context methods
pub trait Context<'llvm> {
    /// Get LLVM context
    fn llvm(&self) -> inkwell::context::ContextRef<'llvm>;

    /// Get LLVM IR for PPL's types
    fn types(&self) -> Types<'llvm> {
        Types::new(self.llvm())
    }

    /// Get LLVM IR for PPL's functions
    fn functions<'m>(&'m self) -> Functions<'llvm, 'm>;
}

/// Context for lowering HIR module to LLVM IR
pub struct ModuleContext<'llvm> {
    /// Currently built module
    pub module: inkwell::module::Module<'llvm>,
    /// Builder for debug info
    pub dibuilder: inkwell::debug_info::DebugInfoBuilder<'llvm>,
    /// Compile unit for debug info
    pub compile_unit: inkwell::debug_info::DICompileUnit<'llvm>,
}

impl<'llvm> ModuleContext<'llvm> {
    /// Initialize context for lowering HIR module to LLVM IR
    pub fn new(module: inkwell::module::Module<'llvm>) -> Self {
        let llvm = module.get_context();
        let debug_metadata_version = llvm.i32_type().const_int(3, false);
        module.add_basic_value_flag(
            "Debug Info Version",
            inkwell::module::FlagBehavior::Warning,
            debug_metadata_version,
        );
        let (dibuilder, compile_unit) = module.create_debug_info_builder(
            true,
            /* language */ inkwell::debug_info::DWARFSourceLanguage::Rust,
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
            module,
            dibuilder,
            compile_unit,
        }
    }
}

impl<'llvm> Context<'llvm> for ModuleContext<'llvm> {
    fn llvm(&self) -> inkwell::context::ContextRef<'llvm> {
        self.module.get_context()
    }

    fn functions<'m>(&'m self) -> Functions<'llvm, 'm> {
        Functions::new(&self.module)
    }
}

/// Context for lowering HIR function to LLVM IR
pub struct FunctionContext<'llvm, 'm> {
    /// Context for lowering HIR module to LLVM IR
    pub module_context: &'m mut ModuleContext<'llvm>,
    /// Currently built function
    pub function: inkwell::values::FunctionValue<'llvm>,
    /// Builder for current function
    pub builder: inkwell::builder::Builder<'llvm>,
    /// Parameters of this function
    pub parameters: HashMap<String, inkwell::values::PointerValue<'llvm>>,
    /// Local variables
    pub variables: HashMap<String, inkwell::values::PointerValue<'llvm>>,
}

impl<'llvm, 'm> FunctionContext<'llvm, 'm> {
    /// Initialize context for lowering HIR function to LLVM IR
    pub fn new(
        module_context: &'m mut ModuleContext<'llvm>,
        function: inkwell::values::FunctionValue<'llvm>,
    ) -> Self {
        let llvm = module_context.llvm();

        let builder = llvm.create_builder();
        let basic_block = llvm.append_basic_block(function, "");
        builder.position_at_end(basic_block);

        Self {
            module_context,
            function,
            builder,
            parameters: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Get LLVM IR for variable
    pub fn get_variable(
        &self,
        variable: &ParameterOrVariable,
    ) -> Option<inkwell::values::PointerValue<'llvm>> {
        match variable {
            ParameterOrVariable::Parameter(p) => self.parameters.get(p.name()).cloned(),
            ParameterOrVariable::Variable(v) => {
                if let Some(var) = self.variables.get(v.name()) {
                    return Some(*var);
                }

                self.module_context
                    .module
                    .get_global(v.name())
                    .map(|v| v.as_pointer_value())
            }
        }
    }

    /// Build a new block for the current function.
    /// Optionally jump to other block.
    /// Doesn't change insert point
    pub fn build_block(
        &mut self,
        name: &str,
        statements: &[Statement],
        jump_to: Option<BasicBlock<'llvm>>,
    ) -> BasicBlock<'llvm> {
        let entry = self.builder.get_insert_block().unwrap();

        let block = self.llvm().append_basic_block(self.function, name);
        self.builder.position_at_end(block);
        for stmt in statements {
            stmt.lower_local_to_ir(self);
        }
        if block.get_terminator().is_none() && jump_to.is_some() {
            self.builder.build_unconditional_branch(jump_to.unwrap());
        }
        self.builder.position_at_end(entry);

        block
    }
}

impl Drop for FunctionContext<'_, '_> {
    fn drop(&mut self) {
        let terminator = self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_terminator());

        if terminator.is_none() {
            self.builder.build_return(None);
        }

        assert!(
            self.function.verify(true),
            "Should never produce invalid functions"
        );
    }
}

impl<'llvm> Context<'llvm> for FunctionContext<'llvm, '_> {
    fn llvm(&self) -> inkwell::context::ContextRef<'llvm> {
        self.module_context.llvm()
    }

    fn functions<'m>(&'m self) -> Functions<'llvm, 'm> {
        self.module_context.functions()
    }
}
