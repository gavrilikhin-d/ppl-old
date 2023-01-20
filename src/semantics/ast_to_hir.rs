use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;

use crate::hir::{self, Module, Type, Typed, ParameterOrVariable, CallKind, FunctionNamePart};
use crate::mutability::Mutable;
use crate::named::HashByName;
use crate::syntax::{Ranged, StringWithOffset};

use super::error::*;
use crate::ast::{self, CallNamePart, If};

/// AST to HIR lowering context
pub struct ASTLoweringContext {
    /// Module, which is being lowered
    pub module: Module,
	/// Stack of functions, which are being lowered
	pub functions_stack: Vec<Arc<hir::FunctionDeclaration>>,
}

impl ASTLoweringContext {
    /// Create new lowering context
    pub fn new(module: hir::Module) -> Self {
        Self {
            module,
			functions_stack: Vec::new(),
		}
    }

    /// Recursively find variable starting from current scope
    pub fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		for f in self.functions_stack.iter().rev() {
			if let Some(param) = f.parameters().find(|p| p.name == name) {
				return Some(param.into());
			}
		}

        let var = self.module.variables.get(name);
        if var.is_some() || self.module.is_builtin {
            var.map(|v| v.value.clone().into())
        }
		else {
			Module::builtin().variables.get(name).map(|v| v.value.clone().into())
		}
	}

    /// Recursively find type starting from current scope
    pub fn find_type(&self, name: &str) -> Option<Type> {
        let ty = self.module.types.get(name);
        if ty.is_some() || self.module.is_builtin {
        	ty.map(|t| t.value.clone().into())
        }
		else {
			Module::builtin().types.get(name).map(|t| t.value.clone().into())
		}
    }

    /// Recursively find type starting from current scope, or return error
    pub fn ty(&self, name: &StringWithOffset) -> Result<Type, UnknownType> {
        let t = self.find_type(&name);
        if t.is_none() {
            return Err(UnknownType {
                name: name.into(),
                at: name.range().into(),
            }
            .into());
        }

        Ok(t.unwrap())
    }

	/// Get builtin "None" type
	pub fn none(&self) -> Type {
		if !self.module.is_builtin {
			return Type::none()
		}

		self.module.types.get("None").unwrap().value.clone().into()
	}

	/// Get builtin "Bool" type
	pub fn bool(&self) -> Type {
		if !self.module.is_builtin {
			return Type::bool()
		}

		self.module.types.get("Bool").unwrap().value.clone().into()
	}

	/// Get builtin "Integer" type
	pub fn integer(&self) -> Type {
		if !self.module.is_builtin {
			return Type::integer()
		}

		self.module.types.get("Integer").unwrap().value.clone().into()
	}

	/// Get builtin "String" type
	pub fn string(&self) -> Type {
		if !self.module.is_builtin {
			return Type::string()
		}

		self.module.types.get("String").unwrap().value.clone().into()
	}

    /// Recursively find all functions with same name format
    pub fn find_functions_with_format(
        &self,
        format: &str,
    ) -> HashSet<HashByName<Arc<hir::FunctionDeclaration>>> {
        let funcs = self
            .module
            .functions
            .get(format)
            .cloned()
            .unwrap_or_default();
		if !self.module.is_builtin {
    		return funcs.union(
				&Module::builtin()
					.functions
						.get(format)
						.cloned()
						.unwrap_or_default()
			).cloned().collect()
		}
        funcs
    }

	/// Get candidates for function call
	pub fn get_candidates(
		&self,
		name_parts: &[CallNamePart],
		args_cache: &[Option<hir::Expression>]
	)
		-> Vec<Arc<hir::FunctionDeclaration>>
	{
		let mut functions = self.module.functions.values().flatten().map(|f| f.value.clone()).collect::<Vec<_>>();

		if !self.module.is_builtin {
			functions.extend(
				Module::builtin().functions.values().flatten().map(|f| f.value.clone())
			)
		}

		let mut candidates = Vec::new();
		for f in functions {
			if f.name_parts.len() != name_parts.len() {
				continue;
			}

			if f.name_parts.iter()
				.zip(name_parts)
				.enumerate()
				.all(
					|(i, (f_part, c_part))| match (f_part, c_part) {
					(FunctionNamePart::Text(text1), CallNamePart::Text(text2)) => text1.as_str() == text2.as_str(),
					(FunctionNamePart::Parameter(_), CallNamePart::Argument(_)) => true,
					(FunctionNamePart::Parameter(_), CallNamePart::Text(_)) => args_cache[i].is_some(),
					_ => false,
				})
			{
				candidates.push(f.clone())
			}
		}
		candidates
	}

	/// Recursively find function with same name format and arguments
	pub fn get_function(
		&self,
		range: Range<usize>,
		format: &str,
		args: &[hir::Expression],
		kind: CallKind,
	) -> Result<Arc<hir::FunctionDeclaration>, Error>
	{
        let functions = self.find_functions_with_format(format);
        let mut name = format.to_string();
        for arg in args {
            name = name.replacen("<>", format!("<:{}>", arg.ty()).as_str(), 1);
        }
        let arguments = args
            .iter()
            .map(|arg| (arg.ty(), arg.range().into()))
            .collect::<Vec<_>>();

        let f = functions.get(name.as_str());
        if f.is_none() {
            let mut candidates: Vec<CandidateNotViable> = Vec::new();
            for candidate in functions {
                for (param, arg) in candidate.value.parameters().zip(args) {
                    if param.ty() != arg.ty() {
                        candidates.push(CandidateNotViable {
                            reason: ArgumentTypeMismatch {
                                expected: param.ty(),
                                expected_span: param.name.range().into(),
                                got: arg.ty(),
                                got_span: arg.range().into(),
                            }
                            .into(),
                        });
                        break;
                    }
                }
            }

			return Err(
				NoFunction {
					kind,
                	name,
                	at: range.into(),
                	arguments,
                	candidates,
            	}.into()
			);
        }

        Ok(f.unwrap().value.clone())
	}
}

impl Default for ASTLoweringContext {
	fn default() -> Self {
		Self::new(Module::default())
	}
}

pub trait ASTLoweringWithinContext {
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error>;
}

impl ASTLoweringWithinContext for ast::Statement {
    type HIR = hir::Statement;

    /// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let stmt: hir::Statement = match self {
            ast::Statement::Declaration(decl) =>
				decl.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Assignment(assign) =>
                assign.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Expression(expr) =>
				expr.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Return(ret) =>
				ret.lower_to_hir_within_context(context)?.into(),
			ast::Statement::If(stmt) =>
				stmt.lower_to_hir_within_context(context)?.into(),
			ast::Statement::Loop(stmt) =>
				stmt.lower_to_hir_within_context(context)?.into(),
        };

        Ok(stmt)
    }
}

impl ASTLoweringWithinContext for ast::Literal {
    type HIR = hir::Literal;

    /// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Literal::None { offset } =>
				hir::Literal::None { offset: *offset, ty: context.none() },
			ast::Literal::Bool { offset, value } =>
				hir::Literal::Bool {
					offset: *offset,
					value: *value,
					ty: context.bool()
				},
            ast::Literal::Integer { value, .. } =>
				hir::Literal::Integer {
					span: self.range(),
					value: value.parse::<rug::Integer>().unwrap(),
					ty: context.integer(),
				},
            ast::Literal::String { value, .. } =>
				hir::Literal::String {
					span: self.range(),
					value: value.clone(),
					ty: context.string(),
				},
        })
    }
}

impl ASTLoweringWithinContext for ast::VariableReference {
    type HIR = hir::VariableReference;

    /// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let var = context.find_variable(&self.name);
        if var.is_none() {
            return Err(UndefinedVariable {
                name: self.name.clone().into(),
                at: self.name.range().into(),
            }
            .into());
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
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let args = vec![
			self.operand.lower_to_hir_within_context(context)?
		];

		let function = context.get_function(
			self.operator.range(),
			self.name_format().as_str(),
			&args,
			CallKind::Operation
		)?;

        Ok(hir::Call {
            function,
            range: self.range().into(),
            args,
        })
    }
}

impl ASTLoweringWithinContext for ast::BinaryOperation {
    type HIR = hir::Call;

    /// Lower [`ast::BinaryOperation`] to [`hir::Call`] within lowering context.
    ///
    /// **Note**: Binary operation is actually a call to function.
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let args = vec![
			self.left.lower_to_hir_within_context(context)?,
			self.right.lower_to_hir_within_context(context)?
		];

		let function = context.get_function(
			self.operator.range(),
			self.name_format().as_str(),
			&args,
			CallKind::Operation
		)?;

        Ok(hir::Call {
            function,
            range: self.range().into(),
            args,
        })
    }
}

impl ASTLoweringWithinContext for ast::Call {
    type HIR = hir::Call;

    /// Lower [`ast::Call`] to [`hir::Call`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
		let args_cache = self.name_parts.iter().map(
			|part| match part {
				CallNamePart::Argument(a) =>
					Ok::<Option<hir::Expression>, Error>(Some(a.lower_to_hir_within_context(context)?)),
				CallNamePart::Text(t) => {
					if let Some(var) = context.find_variable(t) {
						return Ok(Some(
							hir::VariableReference {
								span: t.range().into(),
								variable: var,
							}.into()
						))
					}
					Ok(None)
				}
			}
		).try_collect::<Vec<_>>()?;

		let candidates = context.get_candidates(&self.name_parts, &args_cache);

		let mut candidates_not_viable = Vec::new();
		for f in candidates {
			let mut args = Vec::new();
			let mut failed = false;
			for (i, f_part) in f.name_parts.iter().enumerate() {
				match f_part {
					FunctionNamePart::Text(_) => continue,
					FunctionNamePart::Parameter(p) => {
						let arg = args_cache[i].as_ref().unwrap();
						if arg.ty() != p.ty() {
							candidates_not_viable.push(
								CandidateNotViable {
									reason: ArgumentTypeMismatch {
										expected: p.ty(),
										expected_span: p.name.range().into(),
										got: arg.ty(),
										got_span: arg.range().into()
									}.into()
								}
							);
							failed = true;
							break;
						}
						args.push(arg.clone());
					}
				}
			}

			if !failed {
				return Ok(
					hir::Call {
						range: self.range(),
						function: f.clone(),
						args
					}
				)
			}
		}

		let arguments = args_cache.iter().zip(&self.name_parts).filter_map(
			|(arg, part)| if matches!(part, CallNamePart::Argument(_)) {
				let arg = arg.as_ref().unwrap();
				Some((arg.ty(), arg.range().into()))
			}
			else {
				None
			}
		).collect::<Vec<_>>();

		let mut name = self.name_format().to_string();
        for arg in &arguments {
            name = name.replacen("<>", format!("<:{}>", arg.0).as_str(), 1);
        }
		Err(
			NoFunction {
				kind: CallKind::Call,
				name,
				arguments,
				candidates: candidates_not_viable,
				at: self.range().into()
			}.into()
		)
    }
}

impl ASTLoweringWithinContext for ast::Tuple {
	type HIR = hir::Expression;

	/// Lower [`ast::Tuple`] to [`hir::Expression`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext,
		) -> Result<Self::HIR, Error> {
		if self.expressions.len() == 1 {
			return self.expressions[0].lower_to_hir_within_context(context);
		}
		todo!("real tuples")
	}
}

impl ASTLoweringWithinContext for ast::Expression {
    type HIR = hir::Expression;

    /// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Expression::Literal(l) =>
				l.lower_to_hir_within_context(context)?.into(),
            ast::Expression::VariableReference(var) =>
                var.lower_to_hir_within_context(context)?.into(),
            ast::Expression::UnaryOperation(op) =>
				op.lower_to_hir_within_context(context)?.into(),
            ast::Expression::Call(call) =>
				call.lower_to_hir_within_context(context)?.into(),
			ast::Expression::Tuple(t) =>
				t.lower_to_hir_within_context(context)?.into(),
			ast::Expression::BinaryOperation(op) =>
				op.lower_to_hir_within_context(context)?.into(),
		})
    }
}


/// Trait for lowering conditional expression
trait Condition {
	/// Lower expression that is a condition
	fn lower_condition_to_hir(&self, context: &mut ASTLoweringContext)
		-> Result<hir::Expression, Error>;
}

impl Condition for ast::Expression {
	fn lower_condition_to_hir(&self, context: &mut ASTLoweringContext)
			-> Result<hir::Expression, Error> {
		let condition = self.lower_to_hir_within_context(context)?;
		if condition.ty() != context.bool() {
			return Err(
				ConditionTypeMismatch {
					got: condition.ty(),
					at: condition.range().into()
				}.into()
			);
		}

		Ok(condition)
	}
}

impl ASTLoweringWithinContext for ast::VariableDeclaration {
    type HIR = Arc<hir::VariableDeclaration>;

    /// Lower [`ast::VariableDeclaration`] to [`hir::VariableDeclaration`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let var = Arc::new(hir::VariableDeclaration {
            name: self.name.clone(),
            initializer: self.initializer.lower_to_hir_within_context(context)?,
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
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let ty = Arc::new(hir::TypeDeclaration {
            name: self.name.clone(),
			is_builtin: context.module.is_builtin,
        });

        context.module.types.insert(ty.clone().into());

        Ok(ty)
    }
}

impl ASTLoweringWithinContext for ast::Parameter {
    type HIR = Arc<hir::Parameter>;

    /// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let ty = context.ty(&self.ty)?;

        Ok(Arc::new(hir::Parameter {
            name: self.name.clone(),
            ty,
        }))
    }
}

impl ASTLoweringWithinContext for ast::Annotation {
    type HIR = hir::Annotation;

    /// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        _context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        if self.name == "mangle_as" {
            if let Some(ast::Expression::Literal(ast::Literal::String { value, .. })) =
                self.args.first()
            {
                return Ok(hir::Annotation::MangleAs(value.clone()));
            }
        }
        Err(UnknownAnnotation {
            name: self.name.to_string(),
            at: self.name.range().into(),
        }
        .into())
    }
}

/// Trait to predeclare function in module
trait Predeclare {
	/// Predeclare function in module
	fn predeclare(&self, context: &mut ASTLoweringContext)
		-> Result<Arc<hir::FunctionDeclaration>, Error>;
}

impl Predeclare for ast::FunctionDeclaration {
	fn predeclare(&self, context: &mut ASTLoweringContext)
		-> Result<Arc<hir::FunctionDeclaration>, Error> {
		let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
		for part in &self.name_parts {
			match part {
				ast::FunctionNamePart::Text(t) =>
					name_parts.push(t.clone().into()),
				ast::FunctionNamePart::Parameter(p) => {
					name_parts.push(p.lower_to_hir_within_context(context)?.into())
				}
			}
		}

		let return_type = match &self.return_type {
			Some(ty) => context.ty(ty)?,
			None => context.none(),
		};

		let annotations = self
			.annotations
			.iter()
			.map(|a| a.lower_to_hir_within_context(context))
			.collect::<Result<Vec<_>, _>>()?;
		let mangled_name = annotations.iter().find_map(|a| match a {
			hir::Annotation::MangleAs(name) => Some(name.clone()),
		});

		let f = Arc::new(
			hir::FunctionDeclaration::build()
				.with_name(name_parts.clone())
				.with_mangled_name(mangled_name.clone())
				.with_return_type(return_type.clone()),
		);

		context.module.insert_function(f.clone());

		Ok(f)
	}
}

impl ASTLoweringWithinContext for ast::FunctionDeclaration {
    type HIR = Arc<hir::FunctionDeclaration>;

    /// Lower [`ast::FunctionDeclaration`] to [`hir::FunctionDeclaration`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
       	let f = self.predeclare(context)?;

		context.functions_stack.push(f.clone());
        let mut body = Vec::new();
        for stmt in &self.body {
            body.push(stmt.lower_to_hir_within_context(context)?);
        }
		context.functions_stack.pop();

		let mut return_type = f.return_type.clone();
		if self.implicit_return {
			let expr: hir::Expression = body.pop().unwrap().try_into().unwrap();
			if self.return_type.is_none() {
				return_type = expr.ty();
			}
			else {
				if expr.ty() != return_type {
					return Err(ReturnTypeMismatch {
						expected: return_type.clone(),
						got: expr.ty(),
						got_span: expr.range().into()
					}.into());
				}
			}
			body = vec![hir::Return{ value: Some(expr) }.into()];
		}

        let f = Arc::new(
            hir::FunctionDeclaration::build()
                .with_name(f.name_parts.clone())
                .with_mangled_name(f.mangled_name.clone())
				.with_body(body)
                .with_return_type(return_type),
        );

		context.module.define_function(f.clone());

        Ok(f)
    }
}

impl ASTLoweringWithinContext for ast::Declaration {
    type HIR = hir::Declaration;

    /// Lower [`ast::Declaration`] to [`hir::Declaration`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Declaration::Variable(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Type(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Function(decl) => decl.lower_to_hir_within_context(context)?.into(),
        })
    }
}

impl ASTLoweringWithinContext for ast::Assignment {
    type HIR = hir::Assignment;

    /// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ASTLoweringContext,
    ) -> Result<Self::HIR, Error> {
        let target = self.target.lower_to_hir_within_context(context)?;
        if target.is_immutable() {
            return Err(AssignmentToImmutable {
                at: self.target.range().into(),
            }
            .into());
        }

        let value = self.value.lower_to_hir_within_context(context)?;
        if target.ty() != value.ty() {
            return Err(TypeMismatch {
                got: value.ty(),
                got_span: self.value.range().into(),

                expected: target.ty(),
                expected_span: self.target.range().into(),
            }
            .into());
        }

        Ok(hir::Assignment { target, value })
    }
}

impl ASTLoweringWithinContext for ast::Return {
	type HIR = hir::Return;

	/// Lower [`ast::Return`] to [`hir::Return`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext,
	) -> Result<Self::HIR, Error> {
		let value = self.value.as_ref().map(
			|expr| expr.lower_to_hir_within_context(context)
		).transpose()?;

		if let Some(f) = context.functions_stack.last() {
			if value.is_none() && !f.return_type.is_none() {
				return Err(MissingReturnValue {
					ty: f.return_type.clone(),
					at: self.range().end.into(),
				}.into());
			}
			let value = value.as_ref().unwrap();
			if value.ty() != f.return_type {
				return Err(ReturnTypeMismatch {
					got: value.ty(),
					got_span: value.range().into(),

					expected: f.return_type.clone(),
				}.into());
			}
		}
		else
		{
			return Err(ReturnOutsideFunction {
				at: self.range().into(),
			}.into());
		}

		Ok(hir::Return { value })
	}
}

impl ASTLoweringWithinContext for If {
	type HIR = hir::If;

	/// Lower [`ast::If`] to [`hir::If`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext,
		) -> Result<Self::HIR, Error> {
		Ok(hir::If {
			condition: self.condition.lower_condition_to_hir(context)?,
			body: self.body.iter().map(
				|stmt| stmt.lower_to_hir_within_context(context)
			).try_collect()?,
			else_ifs: self.else_ifs.iter().map(
				|else_if| Ok::<hir::ElseIf, Error>(hir::ElseIf {
					condition:
						else_if.condition.lower_condition_to_hir(context)?,
					body:
						else_if.body.iter().map(
							|stmt| stmt.lower_to_hir_within_context(context)
						).try_collect()?,
				})
			).try_collect()?,
			else_block: self.else_block.iter().map(
				|stmt| stmt.lower_to_hir_within_context(context)
			).try_collect()?,
		})
	}
}

impl ASTLoweringWithinContext for ast::Loop {
	type HIR = hir::Loop;

	/// Lower [`ast::Loop`] to [`hir::Loop`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut ASTLoweringContext,
	) -> Result<Self::HIR, Error> {
		Ok(hir::Loop {
			body: self.body.iter().map(
				|stmt| stmt.lower_to_hir_within_context(context)
			).try_collect()?,
		})
	}
}

impl ASTLoweringWithinContext for ast::Module {
	type HIR = hir::Module;

	/// Lower [`ast::Module`] to [`hir::Module`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ASTLoweringContext,
		) -> Result<Self::HIR, Error> {
		let types = self.statements.iter().filter_map(
			|stmt|
			if let ast::Statement::Declaration(ast::Declaration::Type(_)) = stmt {
				Some(stmt.lower_to_hir_within_context(context))
			}
			else {
				None
			}
		).try_collect::<Vec<_>>()?;

		for stmt in &self.statements {
			if let ast::Statement::Declaration(ast::Declaration::Function(f)) = stmt {
				f.predeclare(context)?;
			}
		}

		let mut current_type = types.iter();
		for stmt in &self.statements {
			let stmt = if let ast::Statement::Declaration(ast::Declaration::Type(_)) = stmt {
				current_type.next().unwrap().clone()
			}
			else {
				stmt.lower_to_hir_within_context(context)?
			};
            context.module.statements.push(stmt);
        }

        Ok(context.module.clone())
    }
}

/// Trait for lowering and adding statements to module
pub trait ASTLowering {
    type HIR;

    /// Lower AST to HIR
    fn lower_to_hir(&self) -> Result<Self::HIR, Error>;
}

impl<T: ASTLoweringWithinContext> ASTLowering for T {
    type HIR = T::HIR;

    /// Lower AST to HIR
    fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
        let mut context = ASTLoweringContext::default();
        self.lower_to_hir_within_context(&mut context)
    }
}