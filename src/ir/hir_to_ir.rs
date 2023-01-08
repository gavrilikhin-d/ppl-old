use std::collections::HashMap;
use std::sync::Arc;

use inkwell::types::BasicMetadataTypeEnum;
use inkwell::AddressSpace;

use inkwell::types::BasicType;
use inkwell::values::BasicMetadataValueEnum;

use crate::hir::*;
use crate::mutability::Mutable;
use crate::named::HashByName;
use crate::named::Named;

/// Trait for lowering HIR for global declarations to IR within module context
pub trait GlobalHIRLowering<'llvm> {
    type IR;

    /// Lower HIR for global declaration to IR within module context
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

/// Trait for lowering HIR for local declarations to IR within function context
pub trait LocalHIRLowering<'llvm, 'm> {
    type IR;

    /// Lower HIR for local declaration to IR within function context
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// Trait for lowering HIR to IR within function context
pub trait HIRLoweringWithinFunctionContext<'llvm, 'm> {
    type IR;

    /// Lower HIR to IR within function context
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// LLVM IR for PPL's types
pub struct Types<'llvm> {
    /// LLVM context
    llvm: inkwell::context::ContextRef<'llvm>,
}

impl<'llvm> Types<'llvm> {
    /// Initialize LLVM IR for PPL's types
    pub(crate) fn new(llvm: inkwell::context::ContextRef<'llvm>) -> Self {
        Self { llvm }
    }

    /// LLVM void type
    pub fn void(&self) -> inkwell::types::VoidType<'llvm> {
        self.llvm.void_type()
    }

    /// LLVM int type
    pub fn i(&self, bits: u32) -> inkwell::types::IntType<'llvm> {
        self.llvm.custom_width_int_type(bits)
    }

    /// LLVM unsigned int type
    pub fn u(&self, bits: u32) -> inkwell::types::IntType<'llvm> {
        self.i(bits)
    }

    /// Get LLVM opaque struct type or create it if it doesn't exist
    fn get_or_add_opaque_struct(&self, name: &str) -> inkwell::types::StructType<'llvm> {
        if let Some(ty) = self.llvm.get_struct_type(name) {
            return ty;
        }

        self.llvm.opaque_struct_type(name)
    }

    /// LLVM IR for [`Class`](Type::Class) type
    pub fn class(&self, name: &str) -> inkwell::types::PointerType<'llvm> {
        self.get_or_add_opaque_struct(name)
            .ptr_type(AddressSpace::Generic)
    }

    /// LLVM IR for [`None`](Type::None) type
    pub fn none(&self) -> inkwell::types::PointerType<'llvm> {
        self.class("None")
    }

    /// LLVM IR for [`Integer`](Type::Integer) type
    pub fn integer(&self) -> inkwell::types::PointerType<'llvm> {
        self.class("Integer")
    }

    /// LLVM IR for [`String`](Type::String) type
    pub fn string(&self) -> inkwell::types::PointerType<'llvm> {
        self.class("String")
    }

    /// LLVM IR for C string type
    pub fn c_string(&self) -> inkwell::types::PointerType<'llvm> {
        self.llvm.i8_type().ptr_type(AddressSpace::Generic)
    }
}

/// Trait for convenient lowering of PPL's [`Type`](Type) to LLVM IR
pub trait HIRTypesLowering<'llvm> {
    type IR;

    /// Lower PPL's [`Type`](Type) to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR;
}

impl<'llvm> HIRTypesLowering<'llvm> for Type {
    type IR = inkwell::types::BasicTypeEnum<'llvm>;

    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
        match self {
            Type::None => context.types().none().into(),
            Type::Integer => context.types().integer().into(),
            Type::String => context.types().string().into(),
            Type::Class(ty) => ty.lower_to_ir(context).into(),
            Type::Function { .. } => unimplemented!("Function type lowering"),
        }
    }
}

// IMPORTANT: don't forget to update global mapping when adding new function!!!
/// LLVM IR for PPL's functions
pub struct Functions<'llvm, 'm> {
    module: &'m inkwell::module::Module<'llvm>,
}

impl<'llvm, 'm> Functions<'llvm, 'm> {
    /// Initialize LLVM IR for PPL's functions
    pub fn new(module: &'m inkwell::module::Module<'llvm>) -> Self {
        Self { module }
    }

    /// Get function by name
    pub fn get(&self, name: &str) -> Option<inkwell::values::FunctionValue<'llvm>> {
        self.module.get_function(name)
    }

    /// Get function by name if it exists, or add a declaration for it
    pub fn get_or_add_function(
        &self,
        name: &str,
        ty: inkwell::types::FunctionType<'llvm>,
    ) -> inkwell::values::FunctionValue<'llvm> {
        if let Some(f) = self.module.get_function(&name) {
            return f;
        }
        self.module.add_function(name, ty, None)
    }

    /// LLVM IR for default constructor of [`None`](Type::None) type
    pub fn none(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function("none", types.none().fn_type(&[], false))
    }

    /// LLVM IR for constructor of [`Integer`](Type::Integer) type from i64
    pub fn integer_from_i64(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_from_i64",
            types.integer().fn_type(&[types.i(64).into()], false),
        )
    }

    /// LLVM IR for constructor of [`Integer`](Type::Integer) type from C string
    pub fn integer_from_c_string(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "integer_from_c_string",
            types.integer().fn_type(&[types.c_string().into()], false),
        )
    }

    /// LLVM IR for constructor of [`String`](Type::String) type from C string
    /// and its length
    pub fn string_from_c_string_and_length(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "string_from_c_string_and_length",
            types
                .string()
                .fn_type(&[types.c_string().into(), types.u(64).into()], false),
        )
    }

    /// LLVM IR for "print <x: None> -> None" builtin function
    pub fn print_integer(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "print_integer",
            types.none().fn_type(&[types.integer().into()], false),
        )
    }

    /// LLVM IR for "print <str: String> -> None" builtin function
    pub fn print_string(&self) -> inkwell::values::FunctionValue<'llvm> {
        let types = Types::new(self.module.get_context());
        self.get_or_add_function(
            "print_string",
            types.none().fn_type(&[types.string().into()], false),
        )
    }

    // IMPORTANT: don't forget to update global mapping when adding new function!!!
}

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
    /// Global variables
    pub variables:
        HashMap<HashByName<Arc<VariableDeclaration>>, inkwell::values::PointerValue<'llvm>>,
}

impl<'llvm> ModuleContext<'llvm> {
    /// Initialize context for lowering HIR module to LLVM IR
    pub fn new(module: inkwell::module::Module<'llvm>) -> Self {
        Self {
            module,
            variables: HashMap::new(),
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

impl<'llvm> GlobalHIRLowering<'llvm> for Declaration {
    type IR = ();

    /// Lower global [`Declaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_global_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                f.lower_global_to_ir(context);
            }
        }
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Declaration {
    type IR = ();

    /// Lower local [`Declaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_local_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                f.lower_local_to_ir(context);
            }
        }
    }
}

/// Trait for declaring global entries in LLVM IR
trait DeclareGlobal<'llvm> {
    type IR;

    /// Declare global value without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

impl<'llvm> DeclareGlobal<'llvm> for VariableDeclaration {
    type IR = inkwell::values::GlobalValue<'llvm>;

    /// Declare global variable without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let ty = self.ty().lower_to_ir(context);
        let global = context.module.add_global(ty, None, &self.name);

        if self.is_immutable() {
            global.set_constant(true);
        }

        global
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Arc<VariableDeclaration> {
    type IR = inkwell::values::GlobalValue<'llvm>;

    /// Lower global [`VariableDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let global = self.declare_global(context);

        context
            .variables
            .insert(self.clone().into(), global.clone().as_pointer_value());

        global.set_initializer(&self.ty().lower_to_ir(context).const_zero());

        let initialize = context.module.add_function(
            "initialize",
            context.llvm().void_type().fn_type(&[], false),
            None,
        );
        let mut f_context = FunctionContext::new(context, initialize);
        let value = self.initializer.lower_to_ir(&mut f_context);
        f_context
            .builder
            .build_store(global.as_pointer_value(), value);

        global
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Arc<VariableDeclaration> {
    type IR = inkwell::values::PointerValue<'llvm>;

    /// Lower local [`VariableDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        unimplemented!("Local variable declaration")
    }
}

impl<'llvm> HIRTypesLowering<'llvm> for TypeDeclaration {
    type IR = inkwell::types::PointerType<'llvm>;

    /// Lower [`TypeDeclaration`] to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
        context.types().class(&self.name)
    }
}

impl<'llvm> DeclareGlobal<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Declare global function without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let ty = match self.ty() {
            Type::Function {
                parameters,
                return_type,
            } => {
                let parameters = parameters
                    .iter()
                    .map(|p| p.lower_to_ir(context).into())
                    .collect::<Vec<BasicMetadataTypeEnum>>();
                let return_type = return_type.lower_to_ir(context);
                return_type.fn_type(&parameters, false)
            }
            _ => unreachable!("FunctionDeclaration::ty() returned non-function type"),
        };
        context.module.add_function(self.mangled_name(), ty, None)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`FunctionDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let f = self.declare_global(context);

        if let Some(stmts) = &self.body {
            let mut f_context = FunctionContext::new(context, f);
            for stmt in stmts {
                stmt.lower_local_to_ir(&mut f_context);
            }
        }

        f
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`FunctionDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        // TODO: limit function visibility, capture variables, etc.
        let f = self.declare_global(context.module_context);

        if let Some(stmts) = &self.body {
            let mut f_context = FunctionContext::new(context.module_context, f);
            for stmt in stmts {
                stmt.lower_local_to_ir(&mut f_context);
            }
        }

        f
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
        }
    }

    /// Get LLVM IR for variable
    pub fn get_variable(
        &self,
        variable: &VariableDeclaration,
    ) -> Option<inkwell::values::PointerValue<'llvm>> {
        self.module_context.variables.get(variable.name()).cloned()
    }
}

impl Drop for FunctionContext<'_, '_> {
    fn drop(&mut self) {
        let terminator = self
            .builder
            .get_insert_block()
            .and_then(|b| b.get_terminator());

        if terminator.is_some() {
            return;
        }

        self.builder.build_return(None);
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

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Literal {
    type IR = inkwell::values::BasicValueEnum<'llvm>;

    /// Lower [`Literal`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Literal::None { .. } => context
                .builder
                .build_call(context.functions().none(), &[], "")
                .try_as_basic_value()
                .left()
                .unwrap(),
            Literal::Integer { value, .. } => {
                if let Some(value) = value.to_i64() {
                    return context
                        .builder
                        .build_call(
                            context.functions().integer_from_i64(),
                            &[context.types().i(64).const_int(value as u64, false).into()],
                            "",
                        )
                        .try_as_basic_value()
                        .left()
                        .unwrap();
                }

                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "");
                context
                    .builder
                    .build_call(
                        context.functions().integer_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::String { value, .. } => {
                let str = context.builder.build_global_string_ptr(&value, "");
                context
                    .builder
                    .build_call(
                        context.functions().string_from_c_string_and_length(),
                        &[
                            str.as_pointer_value().into(),
                            context
                                .types()
                                .u(64)
                                .const_int(value.len() as u64, false)
                                .into(),
                        ],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for VariableReference {
    type IR = inkwell::values::PointerValue<'llvm>;

    /// Lower [`VariableReference`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        if let Some(var) = context.get_variable(&self.variable) {
            return var;
        }

        self.variable
            .declare_global(context.module_context)
            .as_pointer_value()
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Call {
    type IR = inkwell::values::CallSiteValue<'llvm>;

    /// Lower [`Call`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let function = context
            .functions()
            .get(self.function.mangled_name())
            .or_else(|| Some(self.function.declare_global(context.module_context)))
            .unwrap();

        let arguments = self
            .args
            .iter()
            .map(|arg| arg.lower_to_ir(context).into())
            .collect::<Vec<BasicMetadataValueEnum>>();

        context.builder.build_call(function, &arguments, "")
    }
}

/// Trait for [`Expression`] to lower HIR to LLVM IR without loading references
trait HIRExpressionLoweringWithoutLoad<'llvm, 'm> {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> inkwell::values::BasicValueEnum<'llvm>;
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for Expression {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> inkwell::values::BasicValueEnum<'llvm> {
        match self {
            Expression::VariableReference(var) => var.lower_to_ir(context).into(),

            Expression::Literal(l) => l.lower_to_ir(context),
            Expression::Call(call) => call
                .lower_to_ir(context)
                .try_as_basic_value()
                .left()
                .unwrap(),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Expression {
    type IR = inkwell::values::BasicValueEnum<'llvm>;

    /// Lower [`Expression`] to LLVM IR with loading references
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Expression::VariableReference(var) => {
                let var = var.lower_to_ir(context);
                context.builder.build_load(var, "").into()
            }
            _ => self.lower_to_ir_without_load(context),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Assignment {
    type IR = inkwell::values::InstructionValue<'llvm>;

    /// Lower [`Assignment`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let target = self.target.lower_to_ir_without_load(context);
        let value = self.value.lower_to_ir(context);
        context
            .builder
            .build_store(target.into_pointer_value(), value)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Statement {
    type IR = ();

    /// Lower global [`Statement`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_global_to_ir(context),
            Statement::Assignment(a) => {
                let function = context.module.add_function(
                    "execute",
                    context.llvm().void_type().fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);
                a.lower_to_ir(&mut context);
            }
            Statement::Expression(expr) => {
                let function = context.module.add_function(
                    "evaluate",
                    expr.ty().lower_to_ir(context).fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);

                let value = expr.lower_to_ir(&mut context);
                context.builder.build_return(Some(&value));

                function.verify(true);
            }
			Statement::Return(_) =>
				unreachable!("Return statement is not allowed in global scope")
        };
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Statement {
    type IR = ();

    /// Lower local [`Statement`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_local_to_ir(context),
            Statement::Assignment(a) => {
                a.lower_to_ir(context);
            }
            Statement::Expression(expr) => {
                expr.lower_to_ir(context);
            }
			Statement::Return(ret) => {
				let value = ret.value.as_ref().map(
					|expr| expr.lower_to_ir(context)
				);
				if let Some(value) = value {
					context.builder.build_return(Some(&value));
				} else {
					context.builder.build_return(None);
				}
			}
        };
    }
}

/// Trait for lowering HIR Module to LLVM IR
pub trait HIRModuleLowering<'llvm> {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(&self, llvm: &'llvm inkwell::context::Context)
        -> inkwell::module::Module<'llvm>;
}

impl<'llvm> HIRModuleLowering<'llvm> for Module {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(
        &self,
        llvm: &'llvm inkwell::context::Context,
    ) -> inkwell::module::Module<'llvm> {
        let mut context = ModuleContext::new(llvm.create_module(self.name()));

        for statement in &self.statements {
            statement.lower_global_to_ir(&mut context);
        }

        context.module
    }
}
