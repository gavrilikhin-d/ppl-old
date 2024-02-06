use indexmap::IndexMap;

use inkwell::basic_block::BasicBlock;

use crate::{
    hir::{ParameterOrVariable, Statement},
    named::Named,
};

use super::{DebugInfo, Functions, ToIR, Types};

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
    /// Debug information builder
    pub debug_info: DebugInfo<'llvm>,
}

impl<'llvm> ModuleContext<'llvm> {
    /// Initialize context for lowering HIR module to LLVM IR
    pub fn new(module: inkwell::module::Module<'llvm>) -> Self {
        let debug_info = DebugInfo::new(&module);
        Self {
            module,
            debug_info,
        }
    }

    /// Finalize building module
    pub fn take_module(self) -> inkwell::module::Module<'llvm> {
        self.debug_info.finalize();
        self.module
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
    /// Return value of this function
    pub return_value: Option<inkwell::values::PointerValue<'llvm>>,
    /// Basic block for return
    pub return_block: BasicBlock<'llvm>,
    /// Parameters of this function
    pub parameters: IndexMap<String, inkwell::values::PointerValue<'llvm>>,
    /// Local variables
    pub variables: IndexMap<String, inkwell::values::PointerValue<'llvm>>,
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

        let return_type = function.get_type().get_return_type();
        let return_value = return_type.map(|ty| builder.build_alloca(ty, "return_value").unwrap());
        let return_block = llvm.append_basic_block(function, "return");
        builder.position_at_end(return_block);

        let value =
            return_type.map(|ty| builder.build_load(ty, return_value.unwrap(), "").unwrap());
        builder
            .build_return(value.as_ref().map(|v| v as _))
            .unwrap();

        builder.position_at_end(basic_block);

        Self {
            module_context,
            function,
            builder,
            return_value,
            return_block,
            parameters: IndexMap::new(),
            variables: IndexMap::new(),
        }
    }

    /// Get LLVM IR for variable
    pub fn get_variable(
        &self,
        variable: &ParameterOrVariable,
    ) -> Option<inkwell::values::PointerValue<'llvm>> {
        match variable {
            ParameterOrVariable::Parameter(p) => {
                self.parameters.get(&p.name().to_string()).cloned()
            }
            ParameterOrVariable::Variable(v) => {
                if let Some(var) = self.variables.get(&v.name().to_string()) {
                    return Some(*var);
                }

                self.module_context
                    .module
                    .get_global(&v.name())
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
            stmt.to_ir(self);
        }

        let last_block = self.function.get_last_basic_block().unwrap();
        if last_block.get_terminator().is_none() && jump_to.is_some() {
            self.builder.position_at_end(last_block);
            self.builder
                .build_unconditional_branch(jump_to.unwrap())
                .unwrap();
        }
        self.builder.position_at_end(entry);

        block
    }

    /// Build an unconditional branch to return block
    pub fn branch_to_return_block(&mut self) -> inkwell::values::InstructionValue<'llvm> {
        self.builder
            .position_at_end(self.builder.get_insert_block().unwrap());
        self.builder
            .build_unconditional_branch(self.return_block)
            .unwrap()
    }

    /// Load return value, if any and branch
    pub fn load_return_value_and_branch(
        &mut self,
        value: Option<inkwell::values::BasicValueEnum>,
    ) -> inkwell::values::InstructionValue<'llvm> {
        value.map(|v| {
            self.builder
                .build_store(
                    self.return_value
                        .expect("Returning value in a function that doesn't return"),
                    v,
                )
                .unwrap()
        });
        self.branch_to_return_block()
    }
}

impl Drop for FunctionContext<'_, '_> {
    fn drop(&mut self) {
        let terminator = self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_terminator());

        if terminator.is_none() {
            self.branch_to_return_block();
        }

        if !self.function.verify(true) {
            eprintln!("------------------");
            eprintln!("Invalid function:");
            eprintln!("------------------");
            self.function.print_to_stderr();
            eprintln!("");

            eprintln!("------------------");
            eprintln!("Invalid module:");
            eprintln!("------------------");
            self.module_context.module.print_to_stderr();
            eprintln!("");
            panic!("Invalid LLVM IR for function");
        }
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
