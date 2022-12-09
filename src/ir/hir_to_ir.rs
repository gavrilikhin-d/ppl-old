use std::collections::HashMap;

use inkwell::AddressSpace;

use crate::semantics::{self, Typed};
use crate::semantics::hir::*;

struct Context<'ctx> {
	/// LLVM context
	context: inkwell::context::Context,
	/// Currently built module
	module: inkwell::module::Module<'ctx>,
	/// Currently built function
	function: inkwell::values::FunctionValue<'ctx>,
	/// Current variables
	variables: HashMap<VariableDeclaration, inkwell::values::PointerValue<'ctx>>,
	/// Builder for current function
	builder: inkwell::builder::Builder<'ctx>,
}

impl<'ctx> Context<'ctx> {
	/// Get type of [`semantics::Type::None`]
	pub fn none_type(&self) -> inkwell::types::StructType<'ctx> {
		self.context.get_struct_type("None").unwrap()
	}

	/// Get builtin constructor for None
	pub fn none(&self) -> inkwell::values::FunctionValue<'ctx> {
		self.module.get_function("none").unwrap()
	}

	/// Get type of [`semantics::Type::Integer`]
	pub fn integer_type(&self) -> inkwell::types::StructType<'ctx> {
		self.context.get_struct_type("Integer").unwrap()
	}

	/// Get builtin constructor for Integer from C string
	pub fn integer_from_i64(&self) -> inkwell::values::FunctionValue<'ctx> {
		self.module.get_function("integer_from_i64").unwrap()
	}

	/// Get builtin constructor for Integer from C string
	pub fn integer_from_c_string(&self) -> inkwell::values::FunctionValue<'ctx> {
		self.module.get_function("integer_from_c_string").unwrap()
	}

	/// Get type of [`Value`] enum
	pub fn value_type(&self) -> inkwell::types::StructType<'ctx> {
		self.context.get_struct_type("Value").unwrap()
	}

	/// Lower [`Type`](semantics::Type) to LLVM IR
	pub fn lower_type(&self, ty: &semantics::Type) -> inkwell::types::BasicTypeEnum<'ctx> {
		match ty {
			semantics::Type::None => self.none_type().into(),
			semantics::Type::Integer => self.integer_type().into(),
		}
	}

	/// Lower [`Literal`] to LLVM IR
	pub fn lower_literal(&self, literal: &Literal) -> inkwell::values::BasicValueEnum {
		match literal {
			Literal::None { .. } =>
				self.builder.build_call(self.none(), &[], "")
					.try_as_basic_value()
					.left()
					.unwrap(),
			Literal::Integer { value, .. } => {
				if let Some(value) = value.to_i64() {
					return self.builder.build_call(
						self.integer_from_i64(),
						&[self.context.i64_type().const_int(value as u64, false).into()],
						""
					).try_as_basic_value().left().unwrap();
				}

				let str = self.builder.build_global_string_ptr(
					&format!("{}", value), ""
				);
				self.builder.build_call(
					self.integer_from_c_string(),
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
		self.variables.get(&var.variable).unwrap().clone()
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

	/// Lower global [`VariableDeclaration`] to LLVM IR
	pub fn lower_global_variable_declaration(&mut self, var: &VariableDeclaration) -> inkwell::values::PointerValue<'ctx> {
		let ty = self.lower_type(&var.get_type());
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
			Statement::Declaration(d) => { self.lower_global_declaration(d); },
		};
	}
}