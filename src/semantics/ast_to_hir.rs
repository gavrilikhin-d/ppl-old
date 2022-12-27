use std::collections::HashSet;
use std::sync::Arc;

use crate::mutability::Mutable;
use crate::named::HashByName;
use crate::syntax::{Ranged, StringWithOffset};
use crate::hir::{self, Type, Module, Typed};

use super::error::*;
use crate::ast;

/// AST to HIR lowering context
pub struct ASTLoweringContext {
	/// Module, which is being lowered
	pub module: Module,
	/// Builtin module (set to None, when lowering builtin module itself)
	pub builtin: Option<Arc<Module>>,
}

impl ASTLoweringContext {
	/// Create new lowering context
	pub fn new(name: &str) -> Self {
		Self { module: Module::new(name), builtin: Some(Module::builtin()) }
	}

	/// Recursively find variable starting from current scope
	pub fn find_variable(&self, name: &str)
	-> Option<Arc<hir::VariableDeclaration>> {
		let var = self.module.variables.get(name);
		if var.is_some() {
			var
		} else {
			self.builtin.as_ref().and_then(|m| m.variables.get(name))
		}.map(|x| x.value.clone())
	}

	/// Recursively find type starting from current scope
	pub fn find_type(&self, name: &str) -> Option<Type> {
		let ty = self.module.types.get(name);
		if ty.is_some() {
			ty
		} else {
			self.builtin.as_ref().and_then(|m| m.types.get(name))
		}.map(|t| t.value.clone().into())
	}

	/// Recursively find type starting from current scope, or return error
	pub fn ty(&self, name: &StringWithOffset) -> Result<Type, UnknownType> {
		let t = self.find_type(&name);
		if t.is_none() {
			return Err(UnknownType {
				name: name.into(),
				at: name.range().into()
			}.into());
		}

		Ok(t.unwrap())
	}

	/// Recursively find all functions with same name format
	pub fn find_functions_with_format(&self, format: &str)
		-> HashSet<
			HashByName<
				Arc<hir::FunctionDeclaration>
			>
		>
	{
		let funcs = self.module.functions.get(format).cloned().unwrap_or_default();
		let builtins = self.builtin.as_ref().and_then(|m| m.functions.get(format)).cloned().unwrap_or_default();
		funcs.union(&builtins).cloned().collect()
	}
}

pub trait ASTLoweringWithinContext {
	type HIR;

	/// Lower AST to HIR within some context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error>;
}

impl ASTLoweringWithinContext for ast::Statement {
	type HIR = hir::Statement;

	/// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let stmt: hir::Statement =
		match self {
			ast::Statement::Declaration(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Assignment(assign) =>
				assign.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Expression(expr) =>
				expr.lower_to_hir_within_context(context)?.into(),
		};

		context.module.statements.push(stmt.clone());

		Ok(stmt)
	}
}

impl ASTLoweringWithinContext for ast::Literal {
	type HIR = hir::Literal;

	/// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			_context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(match self {
			ast::Literal::None { offset } =>
				hir::Literal::None { offset: *offset },
			ast::Literal::Integer { value, .. } =>
				hir::Literal::Integer {
					span: self.range(),
					value: value.parse::<rug::Integer>().unwrap(),
				},
			ast::Literal::String { value, .. } =>
				hir::Literal::String {
					span: self.range(),
					value: value.clone(),
				},
		})
	}
}

impl ASTLoweringWithinContext for ast::VariableReference {
	type HIR = hir::VariableReference;

	/// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let var = context.find_variable(&self.name);
		if var.is_none() {
			return Err(UndefinedVariable {
				name: self.name.clone().into(),
				at: self.name.range().into()
			}.into());
		}

		Ok(hir::VariableReference {
			span: self.name.range().into(),
			variable: var.unwrap(),
		})
	}
}

impl ASTLoweringWithinContext for ast::UnaryOperation {
	type HIR = hir::Call;

	/// Lower [`ast::UnaryOperation`] to [`hir::Call`] within lowering context.
	///
	/// **Note**: Unary operator is actually a call to function.
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let operand = self.operand.lower_to_hir_within_context(context)?;

		let name_format = self.name_format();
		let functions = context.find_functions_with_format(&name_format);
		if functions.is_empty() {
			return Err(NoUnaryOperator {
				name: name_format.replace(
					"<>",
					format!("<:{}>", operand.ty()).as_str()
				),
				operator_span: self.operator.range().into(),
				operand_type: operand.ty(),
				operand_span: operand.range().into(),
			}.into());
		}
		unimplemented!(
			"Unary operator ast to hir lowering is not implemented yet"
		);
	}
}

impl ASTLoweringWithinContext for ast::Call {
	type HIR = hir::Call;

	/// Lower [`ast::Call`] to [`hir::Call`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let mut args = Vec::new();
		for part in &self.name_parts {
			match part {
				// TODO: some text parts may actually be variable references
				ast::CallNamePart::Text(_) => continue,
				ast::CallNamePart::Argument(arg) => {
					args.push(arg.lower_to_hir_within_context(context)?);
				}
			}
		}

		let name_format = self.name_format();
		let functions = context.find_functions_with_format(&name_format);
		let mut name = name_format;
		for arg in &args {
			name = name.replacen("<>", format!("<:{}>", arg.ty()).as_str(), 1);
		}
		let arguments = args.iter().map(
			|arg| (arg.ty(), arg.range().into())
		).collect::<Vec<_>>();

		let f = functions.get(name.as_str());
		if f.is_none() {
			let mut candidates: Vec<CandidateNotViable> = Vec::new();
			for candidate in functions {
				for (param, arg) in candidate.value.parameters().zip(&args) {
					if param.ty() != arg.ty() {
						candidates.push(
							CandidateNotViable {
								reason: ArgumentTypeMismatch {
									expected: param.ty(),
									expected_span: param.name.range().into(),
									got: arg.ty(),
									got_span: arg.range().into(),
								}.into()
							}
						);
						break;
					}
				}
			}

			return Err(NoFunction {
				name,
				at: self.range().into(),
				arguments,
				candidates
			}.into());
		}

		let function = f.unwrap().value.clone();
		Ok(
			hir::Call {
				function,
				range: self.range().into(),
				args,
			}
		)
	}
}

impl ASTLoweringWithinContext for ast::Expression {
	type HIR = hir::Expression;

	/// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(
			match self {
				ast::Expression::Literal(l) =>
					l.lower_to_hir_within_context(context)?.into(),
				ast::Expression::VariableReference(var) =>
					var.lower_to_hir_within_context(context)?.into(),
				ast::Expression::UnaryOperation(op) =>
					op.lower_to_hir_within_context(context)?.into(),
				ast::Expression::Call(call) =>
					call.lower_to_hir_within_context(context)?.into(),
			}
		)
	}
}

impl ASTLoweringWithinContext for ast::VariableDeclaration {
	type HIR = Arc<hir::VariableDeclaration>;

	/// Lower [`ast::VariableDeclaration`] to [`hir::VariableDeclaration`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let var = Arc::new(hir::VariableDeclaration {
			name: self.name.clone(),
			initializer:
				self.initializer.lower_to_hir_within_context(context)?,
			mutability: self.mutability.clone(),
		});

		context.module.variables.insert(var.clone().into());

		Ok(var)
	}
}

impl ASTLoweringWithinContext for ast::TypeDeclaration {
	type HIR = Arc<hir::TypeDeclaration>;

	/// Lower [`ast::TypeDeclaration`] to [`hir::TypeDeclaration`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let ty = Arc::new(hir::TypeDeclaration {
			name: self.name.clone(),
		});

		context.module.types.insert(ty.clone().into());

		Ok(ty)
	}
}

impl ASTLoweringWithinContext for ast::Parameter {
	type HIR = hir::Parameter;

	/// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let ty = context.ty(&self.ty)?;

		Ok(hir::Parameter {
			name: self.name.clone(),
			ty,
		})
	}
}

impl ASTLoweringWithinContext for ast::Annotation {
	type HIR = hir::Annotation;

	/// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		_context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		if self.name == "mangle_as" {
			if let Some(
				ast::Expression::Literal(
					ast::Literal::String { value, .. })
				) = self.args.first() {
				return Ok(hir::Annotation::MangleAs(value.clone()));
			}
		}
		Err(
			UnknownAnnotation {
				name: self.name.to_string(),
				at: self.name.range().into(),
			}.into()
		)
	}
}

impl ASTLoweringWithinContext for ast::FunctionDeclaration {
	type HIR = Arc<hir::FunctionDeclaration>;

	/// Lower [`ast::FunctionDeclaration`] to [`hir::FunctionDeclaration`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext
	) -> Result<Self::HIR, Error> {
		let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
		for part in &self.name_parts {
			match part {
				ast::FunctionNamePart::Text(t) =>
					name_parts.push(t.clone().into()),
				ast::FunctionNamePart::Parameter(p) =>
					name_parts.push(
						p.lower_to_hir_within_context(context)?.into()
					),
			}
		}

		let return_type = match &self.return_type {
			Some(ty) => context.ty(ty)?,
			None => Type::None,
		};

		let annotations = self.annotations.iter().map(
			|a| a.lower_to_hir_within_context(context)
		).collect::<Result<Vec<_>, _>>()?;
		let mangled_name = annotations.iter().find_map(
			|a| match a {
				hir::Annotation::MangleAs(name) => Some(name.clone()),
			}
		);

		let f = Arc::new(
			hir::FunctionDeclaration::build()
				.with_name(name_parts)
				.with_mangled_name(mangled_name)
				.with_return_type(return_type)
		);



		let functions = context.module.functions.get_mut(f.name_format());
		if let Some(functions) = functions {
			functions.insert(f.clone().into());
		}
		else {
			let mut functions = HashSet::new();
			functions.insert(f.clone().into());
			context.module.functions.insert(
				f.name_format().to_string(),
				functions
			);
		}

		Ok(f)
	}
}

impl ASTLoweringWithinContext for ast::Declaration {
	type HIR = hir::Declaration;

	/// Lower [`ast::Declaration`] to [`hir::Declaration`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		Ok(match self {
			ast::Declaration::Variable(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Declaration::Type(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
			ast::Declaration::Function(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
		})
	}
}

impl ASTLoweringWithinContext for ast::Assignment {
	type HIR = hir::Assignment;

	/// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext
		) -> Result<Self::HIR, Error> {
		let target = self.target.lower_to_hir_within_context(context)?;
		if target.is_immutable() {
			return Err(AssignmentToImmutable {
				at: self.target.range().into()
			}.into());
		}

		let value = self.value.lower_to_hir_within_context(context)?;
		if target.ty() != value.ty() {
			return Err (
				TypeMismatch {
					got: value.ty(),
					got_span: self.value.range().into(),

					expected: target.ty(),
					expected_span: self.target.range().into(),
				}.into()
			);
		}

		Ok(hir::Assignment { target, value, })
	}
}


/// Trait for lowering and adding statements to module
pub trait ASTLowering  {
	type HIR;

	/// Lower AST to HIR
	fn lower_to_hir(&self) -> Result<Self::HIR, Error>;
}

impl<T: ASTLoweringWithinContext> ASTLowering for T {
	type HIR = T::HIR;

	/// Lower AST to HIR
	fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
		let mut context = ASTLoweringContext::new("main");
		self.lower_to_hir_within_context(&mut context)
	}
}

impl ASTLowering for ast::Module {
	type HIR = Module;

	/// Lower [`ast::Module`] to [`semantics::Module`](Module)
	fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
		let mut context = ASTLoweringContext::new("main");

		for stmt in &self.statements {
			stmt.lower_to_hir_within_context(&mut context)?;
		}

		Ok(context.module)
	}
}
