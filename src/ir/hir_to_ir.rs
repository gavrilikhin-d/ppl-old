use std::collections::HashMap;

use inkwell::types::BasicType;

use crate::semantics::{self, Typed};
use crate::semantics::hir::*;

/// LLVM IR for PPL's types
struct Types<'ctx> {
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

	/// Lower PPL's [`Type`](semantics::Type) to LLVM IR
	pub fn lower(&self, ty: &semantics::Type) -> inkwell::types::BasicTypeEnum<'ctx> {
		match ty {
			semantics::Type::None => self.none.into(),
			semantics::Type::Integer => self.integer.into(),
		}
	}
}

/// LLVM IR for PPL's functions
struct Functions<'ctx> {
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

/// Context for lowering HIR module to LLVM IR
struct ModuleContext<'ctx> {
	/// LLVM IR for PPL's types
	types: Types<'ctx>,
	/// LLVM IR for PPL's functions
	functions: Functions<'ctx>,
	/// Currently built module
	module: inkwell::module::Module<'ctx>,
	/// Global variables
	variables: HashMap<
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

	/// Get reference to LLVM context
	pub fn llvm(&self) -> inkwell::context::ContextRef<'ctx> {
		self.module.get_context()
	}

	/// Create `evaluate` function to evaluate single expression
	pub fn create_function_for_expression<'hir, 'm>(
		&'m mut self,
		expression: &'hir Expression
	) -> inkwell::values::FunctionValue<'ctx> {
		let function = self.module.add_function(
			"evaluate",
			self.types.lower(&expression.get_type()).fn_type(&[], false),
			None
		);

		let context = FunctionContext::new(self, function);

		let value = context.lower_expression(expression);
		context.builder.build_return(Some(&value));

		function.verify(true);

		function
	}

	/// Lower global [`VariableDeclaration`] to LLVM IR
	pub fn lower_global_variable_declaration(&mut self, var: &VariableDeclaration) -> inkwell::values::PointerValue<'ctx> {
		let ty = self.types.lower(&var.get_type());
		let global = self.module.add_global(ty, None, &var.name.value);
		self.variables.insert(var.clone(), global.as_pointer_value());
		self.variables.get(var).unwrap().clone()
	}

	/// Lower global [`Declaration`] to LLVM IR
	pub fn lower_global_declaration(&mut self, decl: &Declaration) -> inkwell::values::PointerValue<'ctx> {
		match decl {
			Declaration::VariableDeclaration(var) =>
				self.lower_global_variable_declaration(var)
		}
	}
}

/// Context for lowering HIR function to LLVM IR
struct FunctionContext<'ctx, 'm> {
	/// Context for lowering HIR module to LLVM IR
	module_context: &'m mut ModuleContext<'ctx>,
	/// Currently built function
	function: inkwell::values::FunctionValue<'ctx>,
	/// Builder for current function
	builder: inkwell::builder::Builder<'ctx>,
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

	/// Get reference to LLVM context
	pub fn llvm(&self) -> inkwell::context::ContextRef<'ctx> {
		self.module_context.llvm()
	}

	/// Get LLVM IR for PPL's types
	pub fn types(&self) -> &Types<'ctx> {
		&self.module_context.types
	}

	/// Get LLVM IR for PPL's functions
	pub fn functions(&self) -> &Functions<'ctx> {
		&self.module_context.functions
	}

	/// Get LLVM IR for variable
	pub fn get_variable(&self, variable: &VariableDeclaration) -> Option<inkwell::values::PointerValue<'ctx>> {
		self.module_context.variables.get(variable).cloned()
	}

	/// Lower [`Literal`] to LLVM IR
	pub fn lower_literal(&self, literal: &Literal) -> inkwell::values::BasicValueEnum {
		match literal {
			Literal::None { .. } =>
				self.builder.build_call(self.functions().none, &[], "")
					.try_as_basic_value()
					.left()
					.unwrap(),
			Literal::Integer { value, .. } => {
				if let Some(value) = value.to_i64() {
					return self.builder.build_call(
						self.functions().integer_from_i64,
						&[self.llvm().i64_type().const_int(value as u64, false).into()],
						""
					).try_as_basic_value().left().unwrap();
				}

				let str = self.builder.build_global_string_ptr(
					&format!("{}", value), ""
				);
				self.builder.build_call(
					self.functions().integer_from_c_string,
					&[str.as_pointer_value().into()],
					""
				).try_as_basic_value().left().unwrap()
			}
		}
	}

	/// Lower [`VariableReference`] to LLVM IR
	pub fn lower_variable_reference(&self, var: &VariableReference)
		-> inkwell::values::PointerValue<'ctx>
	{
		self.get_variable(&var.variable).unwrap()
	}

	/// Lower [`Expression`] to LLVM IR without loading variables
	pub fn lower_expression_no_load(&self, expression: &Expression) -> inkwell::values::BasicValueEnum {
		match expression {
			Expression::Literal(l) => self.lower_literal(l),
			Expression::VariableReference(var) => self.lower_variable_reference(var).into(),
		}
	}

	/// Lower [`Expression`] to LLVM IR and load variables
	pub fn lower_expression(&self, expression: &Expression) -> inkwell::values::BasicValueEnum {
		match expression {
			Expression::VariableReference(var) => self.builder.build_load(
				self.lower_variable_reference(var), ""
			).into(),
			_ => self.lower_expression_no_load(expression)
		}
	}

	/// Lower [`Assignment`] to LLVM IR
	pub fn lower_assignment(&'ctx self, assignment: &Assignment) -> inkwell::values::InstructionValue<'ctx> {
		let target = self.lower_expression_no_load(&assignment.target);
		let value = self.lower_expression(&assignment.value);
		self.builder.build_store(target.into_pointer_value(), value)
	}

	/// Lower global [`Statement`] to LLVM IR
	pub fn lower_global_statement(&'ctx mut self, stmt: &Statement) {
		match stmt {
			Statement::Assignment(a) => { self.lower_assignment(a); },
			Statement::Expression(e) => { self.lower_expression(e); },
			Statement::Declaration(d) => {
				self.module_context.lower_global_declaration(d);
			},
		};
	}
}


/// Create a module for evaluation of single [`Expression`]
///
/// This will create a module, named `expression`,
/// with a single function `evaluate` that takes no arguments
/// and returns LLVM IR type corresponding to the type of the expression.
///
/// # Example
/// ```
/// use ppl::syntax::ast;
/// use ppl::semantics::ASTLowering;
/// use ppl::ir::hir_to_ir::create_module_for_expression;
///
/// let ast = "42".parse::<ast::Expression>().unwrap();
/// let hir = ast.lower_to_hir().unwrap();
///
/// let llvm = inkwell::context::Context::create();
/// let module = create_module_for_expression(&hir, &llvm);
///
/// module.print_to_stderr();
/// ```
pub fn create_module_for_expression<'hir, 'ctx>(
	expression: &'hir Expression,
	llvm: &'ctx inkwell::context::Context
) -> inkwell::module::Module<'ctx> {
	let module = llvm.create_module("expression");

	let mut context = ModuleContext::new(module);
	context.create_function_for_expression(expression);

	context.module.verify().unwrap();

	context.module
}