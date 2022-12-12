use std::collections::HashMap;

use inkwell::types::BasicType;

use crate::semantics::{self, Typed};
use crate::semantics::hir::*;

/// Trait for lowering HIR for global declarations to IR within module context
pub trait GlobalHIRLowering<'ctx> {
	type IR;

	/// Lower HIR for global declaration to IR within module context
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'ctx>
	) -> Self::IR;
}

/// Trait for lowering HIR for local declarations to IR within function context
pub trait LocalHIRLowering<'ctx, 'm> {
	type IR;

	/// Lower HIR for local declaration to IR within function context
	fn lower_local_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>
	) -> Self::IR;
}

/// Trait for lowering HIR to IR within function context
pub trait HIRLoweringWithinFunctionContext<'ctx, 'm> {
	type IR;

	/// Lower HIR to IR within function context
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>
	) -> Self::IR;
}

/// LLVM IR for PPL's types
pub struct Types<'ctx> {
	/// LLVM context
	pub llvm: inkwell::context::ContextRef<'ctx>,
	/// LLVM IR for [`None`](semantics::Type::None) type
	pub none: inkwell::types::StructType<'ctx>,
	/// LLVM IR for [`Integer`](semantics::Type::Integer) type
	pub integer: inkwell::types::StructType<'ctx>,
	/// LLVM IR for C string type
	pub c_string: inkwell::types::PointerType<'ctx>,
}

impl<'ctx> Types<'ctx> {
	/// Initialize LLVM IR for PPL's types
	pub fn new(llvm: inkwell::context::ContextRef<'ctx>) -> Self {
		let none = llvm.opaque_struct_type("None");
		let integer = llvm.opaque_struct_type("Integer");
		let c_string = llvm.i8_type().ptr_type(inkwell::AddressSpace::Generic);

		Self {
			llvm,
			none,
			integer,
			c_string,
		}
	}
}

/// Trait for convenient lowering of PPL's [`Type`](semantics::Type) to LLVM IR
pub trait HIRTypesLowering<'ctx> {
	type IR;

	/// Lower PPL's [`Type`](semantics::Type) to LLVM IR
	fn lower_to_ir(&self, context: &dyn Context<'ctx>) -> Self::IR;
}

impl<'ctx> HIRTypesLowering<'ctx> for semantics::Type {
	type IR = inkwell::types::BasicTypeEnum<'ctx>;

	fn lower_to_ir(&self, context: &dyn Context<'ctx>) -> Self::IR {
		match self {
			semantics::Type::None => context.types().none.into(),
			semantics::Type::Integer => context.types().integer.into(),
		}
	}
}

/// LLVM IR for PPL's functions
pub struct Functions<'ctx> {
	/// LLVM IR for default constructor of [`None`](semantics::Type::None) type
	pub none: inkwell::values::FunctionValue<'ctx>,
	/// LLVM IR for constructor of [`Integer`](semantics::Type::Integer) type from i64
	pub integer_from_i64: inkwell::values::FunctionValue<'ctx>,
	/// LLVM IR for constructor of [`Integer`](semantics::Type::Integer) type from C string
	pub integer_from_c_string: inkwell::values::FunctionValue<'ctx>,
}

impl<'ctx> Functions<'ctx> {
	/// Initialize LLVM IR for PPL's functions
	pub fn new(types: &Types<'ctx>, module: &inkwell::module::Module<'ctx>) -> Self {
		let none = module.add_function(
			"none",
			types.none.fn_type(&[], false),
			None
		);
		let integer_from_i64 = module.add_function(
			"integer_from_i64",
			types.integer.fn_type(&[types.llvm.i64_type().into()], false),
			None
		);
		let integer_from_c_string = module.add_function(
			"integer_from_c_string",
			types.integer.fn_type(&[types.c_string.into()], false),
			None
		);

		Self {
			none,
			integer_from_i64,
			integer_from_c_string,
		}
	}
}

/// Trait for common context methods
pub trait Context<'ctx> {
	/// Get LLVM context
	fn llvm(&self) -> inkwell::context::ContextRef<'ctx>;

	/// Get LLVM IR for PPL's types
	fn types(&self) -> &Types<'ctx>;

	/// Get LLVM IR for PPL's functions
	fn functions(&self) -> &Functions<'ctx>;
}

/// Context for lowering HIR module to LLVM IR
pub struct ModuleContext<'ctx> {
	/// LLVM IR for PPL's types
	pub types: Types<'ctx>,
	/// LLVM IR for PPL's functions
	pub functions: Functions<'ctx>,
	/// Currently built module
	pub module: inkwell::module::Module<'ctx>,
	/// Global variables
	pub variables: HashMap<
		VariableDeclaration,
		inkwell::values::PointerValue<'ctx>
	>,
}

impl<'ctx> ModuleContext<'ctx> {
	/// Initialize context for lowering HIR module to LLVM IR
	pub fn new(module: inkwell::module::Module<'ctx>) -> Self {
		let types = Types::new(module.get_context());
		let functions = Functions::new(&types, &module);

		Self {
			types,
			functions,
			module,
			variables: HashMap::new(),
		}
	}
}

impl<'ctx> Context<'ctx> for ModuleContext<'ctx> {
	fn llvm(&self) -> inkwell::context::ContextRef<'ctx> {
		self.module.get_context()
	}

	fn types(&self) -> &Types<'ctx> {
		&self.types
	}

	fn functions(&self) -> &Functions<'ctx> {
		&self.functions
	}
}

impl<'ctx> GlobalHIRLowering<'ctx> for Declaration {
	type IR = inkwell::values::GlobalValue<'ctx>;

	/// Lower global [`Declaration`] to LLVM IR
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'ctx>
	) -> Self::IR {
		match self {
			Declaration::VariableDeclaration(var) =>
				var.lower_global_to_ir(context)
		}
	}
}

impl<'ctx> GlobalHIRLowering<'ctx> for VariableDeclaration {
	type IR = inkwell::values::GlobalValue<'ctx>;

	/// Lower global [`VariableDeclaration`] to LLVM IR
	fn lower_global_to_ir(
			&self,
			context: &mut ModuleContext<'ctx>
	) -> Self::IR {
		let ty = self.get_type().lower_to_ir(context);
		let global = context.module.add_global(ty, None, &self.name.value);
		context.variables.insert(
			self.clone(),
			global.clone().as_pointer_value()
		);
		global
	}
}

/// Context for lowering HIR function to LLVM IR
pub struct FunctionContext<'ctx, 'm> {
	/// Context for lowering HIR module to LLVM IR
	pub module_context: &'m mut ModuleContext<'ctx>,
	/// Currently built function
	pub function: inkwell::values::FunctionValue<'ctx>,
	/// Builder for current function
	pub builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx, 'm> FunctionContext<'ctx, 'm> {
	/// Initialize context for lowering HIR function to LLVM IR
	pub fn new(
		module_context: &'m mut ModuleContext<'ctx>,
		function: inkwell::values::FunctionValue<'ctx>
	) -> Self {
		let llvm = module_context.llvm();

		let builder = llvm.create_builder();
		let basic_block = llvm.append_basic_block(function, "");
		builder.position_at_end(basic_block);

		Self {
			module_context,
			function,
			builder
		}
	}

	/// Get LLVM IR for variable
	pub fn get_variable(&self, variable: &VariableDeclaration) -> Option<inkwell::values::PointerValue<'ctx>> {
		self.module_context.variables.get(variable).cloned()
	}
}

impl<'ctx> Context<'ctx> for FunctionContext<'ctx, '_> {
	fn llvm(&self) -> inkwell::context::ContextRef<'ctx> {
		self.module_context.llvm()
	}

	fn types(&self) -> &Types<'ctx> {
		self.module_context.types()
	}

	fn functions(&self) -> &Functions<'ctx> {
		self.module_context.functions()
	}
}

impl<'ctx, 'm> HIRLoweringWithinFunctionContext<'ctx, 'm> for Literal {
	type IR = inkwell::values::BasicValueEnum<'ctx>;

	/// Lower [`Literal`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>
	) -> Self::IR {
		match self {
			Literal::None { .. } =>
				context.builder.build_call(context.functions().none, &[], "")
					.try_as_basic_value()
					.left()
					.unwrap(),
			Literal::Integer { value, .. } => {
				if let Some(value) = value.to_i64() {
					return context.builder.build_call(
						context.functions().integer_from_i64,
						&[context.llvm().i64_type().const_int(value as u64, false).into()],
						""
					).try_as_basic_value().left().unwrap();
				}

				let str = context.builder.build_global_string_ptr(
					&format!("{}", value), ""
				);
				context.builder.build_call(
					context.functions().integer_from_c_string,
					&[str.as_pointer_value().into()],
					""
				).try_as_basic_value().left().unwrap()
			}
		}
	}
}

impl<'ctx, 'm> HIRLoweringWithinFunctionContext<'ctx, 'm> for VariableReference {
	type IR = inkwell::values::PointerValue<'ctx>;

	/// Lower [`VariableReference`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>
	) -> Self::IR {
		context.get_variable(&self.variable).unwrap()
	}
}

/// Trait for [`Expression`] to lower HIR to LLVM IR without loading references
trait HIRExpressionLoweringWithoutLoad<'ctx, 'm> {
	/// Lower [`Expression`] to LLVM IR without loading variables
	fn lower_to_ir_without_load(
		&self,
		context: &mut FunctionContext<'ctx, 'm>,
	) -> inkwell::values::BasicValueEnum<'ctx>;
}

impl<'ctx, 'm> HIRExpressionLoweringWithoutLoad<'ctx, 'm> for Expression {
	/// Lower [`Expression`] to LLVM IR without loading variables
	fn lower_to_ir_without_load(
		&self,
		context: &mut FunctionContext<'ctx, 'm>,
	) -> inkwell::values::BasicValueEnum<'ctx> {
		match self {
			Expression::Literal(l) => l.lower_to_ir(context),
			Expression::VariableReference(var) =>
				var.lower_to_ir(context).into(),
		}
	}
}

impl<'ctx, 'm> HIRLoweringWithinFunctionContext<'ctx, 'm> for Expression {
	type IR = inkwell::values::BasicValueEnum<'ctx>;

	/// Lower [`Expression`] to LLVM IR with loading references
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>,
	) -> Self::IR {
		match self {
			Expression::VariableReference(var) => {
				let var = var.lower_to_ir(context);
				context.builder.build_load(var, "").into()
			}
			_ => self.lower_to_ir_without_load(context),
		}
	}
}

impl<'ctx, 'm> HIRLoweringWithinFunctionContext<'ctx, 'm> for Assignment {
	type IR = inkwell::values::InstructionValue<'ctx>;

	/// Lower [`Assignment`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'ctx, 'm>
	) -> Self::IR {
		let target = self.target.lower_to_ir_without_load(context);
		let value = self.value.lower_to_ir(context);
		context.builder.build_store(target.into_pointer_value(), value)
	}
}

impl<'ctx> GlobalHIRLowering<'ctx> for Statement {
	type IR = inkwell::values::GlobalValue<'ctx>;

	/// Lower global [`Statement`] to LLVM IR
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'ctx>
	) -> Self::IR {
		return match self {
			Statement::Declaration(d) =>
				d.lower_global_to_ir(context),
			Statement::Assignment(a) => {
				let function = context.module.add_function(
					"execute",
					context.llvm().void_type().fn_type(&[], false),
					None
				);

				let mut context = FunctionContext::new(context, function);
				a.lower_to_ir(&mut context);

				function.as_global_value()
			},
			Statement::Expression(expr) =>
			{
				let function = context.module.add_function(
					"evaluate",
					expr.get_type().lower_to_ir(context).fn_type(&[], false),
					None
				);

				let mut context = FunctionContext::new(context, function);

				let value = expr.lower_to_ir(&mut context);
				context.builder.build_return(Some(&value));

				function.verify(true);

				function.as_global_value()
			},
		};
	}
}