use std::collections::HashMap;

use inkwell::types::BasicType;

use crate::semantics::{self, Typed};
use crate::semantics::hir::*;

/// Trait for lowering HIR for global declarations to IR within module context
pub trait GlobalHIRLowering<'llvm> {
	type IR;

	/// Lower HIR for global declaration to IR within module context
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'llvm>
	) -> Self::IR;
}

/// Trait for lowering HIR for local declarations to IR within function context
pub trait LocalHIRLowering<'llvm, 'm> {
	type IR;

	/// Lower HIR for local declaration to IR within function context
	fn lower_local_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>
	) -> Self::IR;
}

/// Trait for lowering HIR to IR within function context
pub trait HIRLoweringWithinFunctionContext<'llvm, 'm> {
	type IR;

	/// Lower HIR to IR within function context
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>
	) -> Self::IR;
}

/// LLVM IR for PPL's types
pub struct Types<'llvm> {
	/// LLVM context
	pub llvm: inkwell::context::ContextRef<'llvm>,
	/// LLVM IR for [`None`](semantics::Type::None) type
	pub none: inkwell::types::StructType<'llvm>,
	/// LLVM IR for [`Integer`](semantics::Type::Integer) type
	pub integer: inkwell::types::StructType<'llvm>,
	/// LLVM IR for C string type
	pub c_string: inkwell::types::PointerType<'llvm>,
}

impl<'llvm> Types<'llvm> {
	/// Initialize LLVM IR for PPL's types
	pub fn new(llvm: inkwell::context::ContextRef<'llvm>) -> Self {
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
pub trait HIRTypesLowering<'llvm> {
	type IR;

	/// Lower PPL's [`Type`](semantics::Type) to LLVM IR
	fn lower_to_ir(&self, context: &dyn Context<'llvm>) -> Self::IR;
}

impl<'llvm> HIRTypesLowering<'llvm> for semantics::Type {
	type IR = inkwell::types::BasicTypeEnum<'llvm>;

	fn lower_to_ir(&self, context: &dyn Context<'llvm>) -> Self::IR {
		match self {
			semantics::Type::None => context.types().none.into(),
			semantics::Type::Integer => context.types().integer.into(),
		}
	}
}

/// LLVM IR for PPL's functions
pub struct Functions<'llvm> {
	/// LLVM IR for default constructor of [`None`](semantics::Type::None) type
	pub none: inkwell::values::FunctionValue<'llvm>,
	/// LLVM IR for constructor of [`Integer`](semantics::Type::Integer) type from i64
	pub integer_from_i64: inkwell::values::FunctionValue<'llvm>,
	/// LLVM IR for constructor of [`Integer`](semantics::Type::Integer) type from C string
	pub integer_from_c_string: inkwell::values::FunctionValue<'llvm>,
}

impl<'llvm> Functions<'llvm> {
	/// Initialize LLVM IR for PPL's functions
	pub fn new(types: &Types<'llvm>, module: &inkwell::module::Module<'llvm>) -> Self {
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
pub trait Context<'llvm> {
	/// Get LLVM context
	fn llvm(&self) -> inkwell::context::ContextRef<'llvm>;

	/// Get LLVM IR for PPL's types
	fn types(&self) -> &Types<'llvm>;

	/// Get LLVM IR for PPL's functions
	fn functions(&self) -> &Functions<'llvm>;
}

/// Context for lowering HIR module to LLVM IR
pub struct ModuleContext<'llvm> {
	/// LLVM IR for PPL's types
	pub types: Types<'llvm>,
	/// LLVM IR for PPL's functions
	pub functions: Functions<'llvm>,
	/// Currently built module
	pub module: inkwell::module::Module<'llvm>,
	/// Global variables
	pub variables: HashMap<
		VariableDeclaration,
		inkwell::values::PointerValue<'llvm>
	>,
}

impl<'llvm> ModuleContext<'llvm> {
	/// Initialize context for lowering HIR module to LLVM IR
	pub fn new(module: inkwell::module::Module<'llvm>) -> Self {
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

impl<'llvm> Context<'llvm> for ModuleContext<'llvm> {
	fn llvm(&self) -> inkwell::context::ContextRef<'llvm> {
		self.module.get_context()
	}

	fn types(&self) -> &Types<'llvm> {
		&self.types
	}

	fn functions(&self) -> &Functions<'llvm> {
		&self.functions
	}
}

impl<'llvm> GlobalHIRLowering<'llvm> for Declaration {
	type IR = inkwell::values::GlobalValue<'llvm>;

	/// Lower global [`Declaration`] to LLVM IR
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'llvm>
	) -> Self::IR {
		match self {
			Declaration::VariableDeclaration(var) =>
				var.lower_global_to_ir(context)
		}
	}
}

impl<'llvm> GlobalHIRLowering<'llvm> for VariableDeclaration {
	type IR = inkwell::values::GlobalValue<'llvm>;

	/// Lower global [`VariableDeclaration`] to LLVM IR
	fn lower_global_to_ir(
			&self,
			context: &mut ModuleContext<'llvm>
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
		function: inkwell::values::FunctionValue<'llvm>
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
	pub fn get_variable(&self, variable: &VariableDeclaration) -> Option<inkwell::values::PointerValue<'llvm>> {
		self.module_context.variables.get(variable).cloned()
	}
}

impl<'llvm> Context<'llvm> for FunctionContext<'llvm, '_> {
	fn llvm(&self) -> inkwell::context::ContextRef<'llvm> {
		self.module_context.llvm()
	}

	fn types(&self) -> &Types<'llvm> {
		self.module_context.types()
	}

	fn functions(&self) -> &Functions<'llvm> {
		self.module_context.functions()
	}
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Literal {
	type IR = inkwell::values::BasicValueEnum<'llvm>;

	/// Lower [`Literal`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>
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

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for VariableReference {
	type IR = inkwell::values::PointerValue<'llvm>;

	/// Lower [`VariableReference`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>
	) -> Self::IR {
		context.get_variable(&self.variable).unwrap()
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
			Expression::Literal(l) => l.lower_to_ir(context),
			Expression::VariableReference(var) =>
				var.lower_to_ir(context).into(),
		}
	}
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Expression {
	type IR = inkwell::values::BasicValueEnum<'llvm>;

	/// Lower [`Expression`] to LLVM IR with loading references
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>,
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

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Assignment {
	type IR = inkwell::values::InstructionValue<'llvm>;

	/// Lower [`Assignment`] to LLVM IR
	fn lower_to_ir(
		&self,
		context: &mut FunctionContext<'llvm, 'm>
	) -> Self::IR {
		let target = self.target.lower_to_ir_without_load(context);
		let value = self.value.lower_to_ir(context);
		context.builder.build_store(target.into_pointer_value(), value)
	}
}

impl<'llvm> GlobalHIRLowering<'llvm> for Statement {
	type IR = inkwell::values::GlobalValue<'llvm>;

	/// Lower global [`Statement`] to LLVM IR
	fn lower_global_to_ir(
		&self,
		context: &mut ModuleContext<'llvm>
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