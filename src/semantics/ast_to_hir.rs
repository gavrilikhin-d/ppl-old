use crate::syntax::{ast, Ranged};
use super::hir::Mutable;
use super::{hir, Module, Typed};

use super::error::*;

/// Trait for lowering and adding statements to module
pub trait Lowering<'m> {
	/// Lower statement and add it to the end of the module
	fn add(&'m mut self, statement: &ast::Statement) -> Result<hir::Statement, Error>;
}

impl<'m> Lowering<'m> for Module {
	/// Lower statement and add it to the end of the module
	fn add(&'m mut self, statement: &ast::Statement) -> Result<hir::Statement, Error> {
		let mut context = Context { module: self };
		let lowered = context.lower_statement(statement);
		lowered
	}
}

/// AST to HIR lowering context
struct Context<'m> {
	/// Module, which is being lowered
	pub module: &'m mut Module,
}

impl Context<'_> {
	/// Recursively find variable starting from current scope
	pub fn find_variable(&self, name: &str) -> Option<& hir::VariableDeclaration> {
		self.module.variables.get(name)
	}

	/// Lower AST for literal to literal
	pub fn lower_literal(&self, literal: &ast::Literal) -> hir::Literal {
		match literal {
			ast::Literal::None { offset } =>
				hir::Literal::None { offset: *offset },
			ast::Literal::Integer { offset, value } =>
				hir::Literal::Integer {
					span: *offset..offset + value.len(),
					value: value.parse::<rug::Integer>().unwrap(),
				},
		}
	}

	/// Lower reference to variable to HIR
	pub fn lower_variable_reference(&self, var_ast: &ast::VariableReference) -> Result<hir::VariableReference, Error> {
		let var = self.find_variable(&var_ast.name.value);
		if var.is_none() {
			return Err(UndefinedVariable {
				name: var_ast.name.value.clone(),
				at: var_ast.name.range().into()
			}.into());
		}

		Ok(hir::VariableReference {
			span: var_ast.name.range().into(),
			variable: Box::new(var.unwrap().clone()),
		})
	}

	/// Lower AST for expression to HIR
	pub fn lower_expression(&self, expr: &ast::Expression) -> Result<hir::Expression, Error> {
		Ok(
			match expr {
				ast::Expression::Literal(l) => self.lower_literal(l).into(),
				ast::Expression::VariableReference(var) =>
					self.lower_variable_reference(var)?.into(),
				ast::Expression::UnaryOperation(op) => unimplemented!()
			}
		)
	}

	/// Lower variable declaration
	pub fn lower_variable_declaration(
		&mut self,
		decl_ast: &ast::VariableDeclaration
	) -> Result<hir::VariableDeclaration, Error> {
		let var = hir::VariableDeclaration {
			name: decl_ast.name.clone(),
			initializer: self.lower_expression(&decl_ast.initializer)?,
			mutability: decl_ast.mutability.clone(),
		};

		let name = &decl_ast.name.value;

		self.module.variables.insert(name.to_owned(), var.clone());

		Ok(var)
	}

	/// Lower AST for declaration to HIR
	pub fn lower_declaration(&mut self, decl_ast: &ast::Declaration) -> Result<hir::Declaration, Error> {
		Ok(
			match decl_ast {
				ast::Declaration::Variable(decl) =>
					self.lower_variable_declaration(decl)?.into(),
			}
		)
	}

	/// Lower AST for assignment to HIR
	pub fn lower_assignment(&self, assign_ast: &ast::Assignment) -> Result<hir::Assignment, Error> {
		let target = self.lower_expression(&assign_ast.target)?;
		if target.is_immutable() {
			return Err(AssignmentToImmutable {
				at: assign_ast.target.range().into()
			}.into());
		}

		let value = self.lower_expression(&assign_ast.value)?;
		if target.get_type() != value.get_type() {
			return Err (
				NoConversion {
					from: value.get_type(),
					from_span: assign_ast.value.range().into(),

					to: target.get_type(),
					to_span: assign_ast.target.range().into(),
				}.into()
			);
		}

		Ok(hir::Assignment { target, value, })
	}

	/// Lower AST for statement to HIR
	pub fn lower_statement(&mut self, stmt_ast: &ast::Statement) -> Result<hir::Statement, Error> {
		let stmt: hir::Statement =
			match &stmt_ast {
				ast::Statement::Declaration(decl) =>
					self.lower_declaration(decl)?.into(),
				ast::Statement::Assignment(assign) =>
					self.lower_assignment(assign)?.into(),
				ast::Statement::Expression(expr) =>
					self.lower_expression(expr)?.into(),
			};

		self.module.statements.push(stmt.clone());

		Ok(stmt)
	}
}

