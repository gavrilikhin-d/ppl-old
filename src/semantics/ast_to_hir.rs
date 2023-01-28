use std::sync::{Arc, Weak};

use crate::hir::{self, Typed, CallKind, FunctionNamePart, FunctionDefinition, Type};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;

use super::{error::*, Context, ModuleContext, FunctionContext, TraitContext};
use crate::ast::{self, CallNamePart, If};

/// Lower AST inside some context
pub trait ASTLoweringWithinContext {
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error>;
}

pub trait ASTLoweringWithinModule {
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(
        &self,
        context: &mut ModuleContext,
    ) -> Result<Self::HIR, Error>;
}

impl ASTLoweringWithinContext for ast::Statement {
    type HIR = hir::Statement;

    /// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
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
			ast::Statement::While(stmt) =>
				stmt.lower_to_hir_within_context(context)?.into(),
        })
    }
}

impl ASTLoweringWithinContext for ast::Literal {
    type HIR = hir::Literal;

    /// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Literal::None { offset } =>
				hir::Literal::None {
					offset: *offset,
					ty: context.builtin().types().none()
				},
			ast::Literal::Bool { offset, value } =>
				hir::Literal::Bool {
					offset: *offset,
					value: *value,
					ty: context.builtin().types().bool()
				},
            ast::Literal::Integer { value, .. } =>
				hir::Literal::Integer {
					span: self.range(),
					value: value.parse::<rug::Integer>().unwrap(),
					ty: context.builtin().types().integer(),
				},
            ast::Literal::String { value, .. } =>
				hir::Literal::String {
					span: self.range(),
					value: value.clone(),
					ty: context.builtin().types().string(),
				},
        })
    }
}

impl ASTLoweringWithinContext for ast::VariableReference {
    type HIR = hir::VariableReference;

    /// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
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
        context: &mut impl Context,
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
            function: function.declaration(),
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
        context: &mut impl Context,
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
            function: function.declaration(),
            range: self.range().into(),
            args,
        })
    }
}

/// Trait to check if one type is convertible to another within context
trait ConvertibleTo {
	/// Is this type convertible to another?
	fn convertible_to(&self, ty: hir::Type) -> ConvertibleToCheck;
}

impl ConvertibleTo for hir::Type {
	fn convertible_to(&self, ty: hir::Type) -> ConvertibleToCheck {
		ConvertibleToCheck {
			from: self.clone(),
			to: ty
		}
	}
}

/// Helper struct to perform check within context
struct ConvertibleToCheck {
	from: Type,
	to: Type
}

impl ConvertibleToCheck {
	pub fn within(&self, context: &impl Context) -> bool {
		match (&self.from, &self.to) {
			(Type::Trait(tr), Type::SelfType(s))
				=> Arc::ptr_eq(&tr, &s.associated_trait.upgrade().unwrap()),
			// TODO: this needs context of all visible functions to check if class implements trait
			(Type::Class(c), Type::Trait(tr))
				=> c.implements(tr.clone()).within(context),
			_ => self.from == self.to
		}
	}
}

/// Trait to check if type implements trait
trait Implements {
	/// Does this class implement given trait?
	fn implements(&self, tr: Arc<hir::TraitDeclaration>) -> ImplementsCheck;
}

impl Implements for Arc<hir::TypeDeclaration> {
	fn implements(&self, tr: Arc<hir::TraitDeclaration>) -> ImplementsCheck {
		ImplementsCheck {
			ty: self.clone().into(),
			tr: tr
		}
	}
}

/// Helper struct to do check within context
struct ImplementsCheck {
	ty: Type,
	tr: Arc<hir::TraitDeclaration>
}

impl ImplementsCheck {
	pub fn within(&self, context: &impl Context) -> bool {
		self.tr.functions.iter().all(
			|f| context.find_implementation(&f, &self.ty).is_some()
		)
	}
}



impl ASTLoweringWithinContext for ast::Call {
    type HIR = hir::Call;

    /// Lower [`ast::Call`] to [`hir::Call`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
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

		let candidates = context.candidates(&self.name_parts, &args_cache);

		let mut candidates_not_viable = Vec::new();
		for f in candidates {
			let mut args = Vec::new();
			let mut failed = false;
			for (i, f_part) in f.name_parts().iter().enumerate() {
				match f_part {
					FunctionNamePart::Text(_) => continue,
					FunctionNamePart::Parameter(p) => {
						let arg = args_cache[i].as_ref().unwrap();
						if !arg.ty().convertible_to(p.ty()).within(context) {
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
				let function = if f.is_generic() {
					context.monomorphize(&f, &args)
				}
				else
				{
					f.declaration()
				};
				return Ok(
					hir::Call {
						range: self.range(),
						function,
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
			context: &mut impl Context,
		) -> Result<Self::HIR, Error> {
		if self.expressions.len() == 1 {
			return self.expressions[0].lower_to_hir_within_context(context);
		}
		todo!("real tuples")
	}
}

impl ASTLoweringWithinContext for ast::TypeReference {
	type HIR = hir::TypeReference;

	/// Lower [`ast::TypeReference`] to [`hir::TypeReference`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut impl Context,
		) -> Result<Self::HIR, Error> {
		let ty = context.find_type(&self.name);
		if ty.is_none() {
			return Err(
				UnknownType {
					name: self.name.clone().into(),
					at: self.name.range().into(),
				}.into()
			);
		}
		Ok(
			hir::TypeReference {
				span: self.range().into(),
				referenced_type: ty.unwrap(),
			}
		)
	}
}

impl ASTLoweringWithinContext for ast::Expression {
    type HIR = hir::Expression;

    /// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
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
			ast::Expression::TypeReference(t) =>
				t.lower_to_hir_within_context(context)?.into(),
		})
    }
}


/// Trait for lowering conditional expression
trait Condition {
	/// Lower expression that is a condition
	fn lower_condition_to_hir(&self, context: &mut impl Context)
		-> Result<hir::Expression, Error>;
}

impl Condition for ast::Expression {
	fn lower_condition_to_hir(&self, context: &mut impl Context)
			-> Result<hir::Expression, Error> {
		let condition = self.lower_to_hir_within_context(context)?;
		if !condition.ty().is_bool() {
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
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        let var = Arc::new(hir::VariableDeclaration {
            name: self.name.clone(),
            initializer: self.initializer.lower_to_hir_within_context(context)?,
            mutability: self.mutability.clone(),
        });

        context.add_variable(var.clone());

        Ok(var)
    }
}

impl ASTLoweringWithinContext for ast::TypeDeclaration {
    type HIR = Arc<hir::TypeDeclaration>;

    /// Lower [`ast::TypeDeclaration`] to [`hir::TypeDeclaration`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        let ty = Arc::new(hir::TypeDeclaration {
            name: self.name.clone(),
			is_builtin: context.is_for_builtin_module()
        });

        context.add_type(ty.clone());

        Ok(ty)
    }
}

impl ASTLoweringWithinContext for ast::Parameter {
    type HIR = Arc<hir::Parameter>;

    /// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        Ok(Arc::new(hir::Parameter {
            name: self.name.clone(),
            ty: self.ty.lower_to_hir_within_context(context)?.referenced_type
        }))
    }
}

impl ASTLoweringWithinContext for ast::Annotation {
    type HIR = hir::Annotation;

    /// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        _context: &mut impl Context,
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
trait Declare {
	/// Declare function in module
	fn declare(&self, context: &mut impl Context)
		-> Result<Arc<hir::FunctionDeclaration>, Error>;
}

impl Declare for ast::FunctionDeclaration {
	fn declare(&self, context: &mut impl Context)
		-> Result<Arc<hir::FunctionDeclaration>, Error> {
		let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
		for part in &self.name_parts {
			match part {
				ast::FunctionNamePart::Text(t) =>
					name_parts.push(t.clone().into()),
				ast::FunctionNamePart::Parameter{parameter, ..} => {
					name_parts.push(parameter.lower_to_hir_within_context(context)?.into())
				}
			}
		}

		let return_type = match &self.return_type {
			Some(ty) =>
				ty.lower_to_hir_within_context(context)?.referenced_type,
			None => context.builtin().types().none(),
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

		context.add_function(f.clone().into());

		Ok(f)
	}
}

trait Define {
	/// Define function in module
	fn define(&self, declaration: Arc<hir::FunctionDeclaration>, context: &mut impl Context)
		-> Result<hir::Function, Error>;
}

impl Define for ast::FunctionDeclaration {
	fn define(
		&self,
		declaration: Arc<hir::FunctionDeclaration>,
		context: &mut impl Context
	) -> Result<hir::Function, Error> {
		if self.body.is_empty() {
			return Ok(declaration.into())
		}

		let mut f_context = FunctionContext {
			function: FunctionDefinition { declaration, body: vec![] },
			parent: context,
		};

		let mut body = Vec::new();
		for stmt in &self.body {
			body.push(stmt.lower_to_hir_within_context(&mut f_context)?);
		}

		if self.implicit_return {
			let return_type = f_context.function.return_type().clone();
			let expr: hir::Expression = body.pop().unwrap().try_into().unwrap();
			if self.return_type.is_none() {
				// One reference is held by module
				// Another reference is held by f itself
				if Arc::strong_count(&f_context.function.declaration) > 2 {
					return Err(CantDeduceReturnType {
						at: self.name_parts.range().into()
					}.into());
				}
				else {
					unsafe {
						(*Arc::as_ptr(&f_context.function.declaration).cast_mut()).return_type = expr.ty().clone();
					}
				}
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

		let f = Arc::new(f_context.function);

		context.add_function(f.clone().into());

		Ok(f.into())
	}
}

impl ASTLoweringWithinContext for ast::FunctionDeclaration {
    type HIR = hir::Function;

    /// Lower [`ast::FunctionDeclaration`] to [`hir::Function`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
		self.define(self.declare(context)?, context)
    }
}

impl ASTLoweringWithinContext for ast::TraitDeclaration {
	type HIR = Arc<hir::TraitDeclaration>;

	/// Lower [`ast::TraitDeclaration`] to [`hir::TraitDeclaration`] within lowering context
	fn lower_to_hir_within_context(
		&self,
		context: &mut impl Context,
	) -> Result<Self::HIR, Error> {
		let mut error = None;
		let tr = Arc::new_cyclic(
			|trait_weak| {
				let mut context = TraitContext {
					tr: hir::TraitDeclaration {
						name: self.name.clone(),
						functions: vec![]
					},
					trait_weak: trait_weak.clone(),
					parent: context,
				};

				for f in &self.functions {
					error = f.lower_to_hir_within_context(&mut context).err();
					if error.is_some() {
						break;
					}
				}

				context.tr
			}
		);

		if let Some(error) = error {
			return Err(error);
		}

        context.add_trait(tr.clone());

        Ok(tr)
	}
}

impl ASTLoweringWithinContext for ast::Declaration {
    type HIR = hir::Declaration;

    /// Lower [`ast::Declaration`] to [`hir::Declaration`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Declaration::Variable(decl)
				=> decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Type(decl)
				=> decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Function(decl)
				=> decl.lower_to_hir_within_context(context)?.into(),
			ast::Declaration::Trait(decl)
				=> decl.lower_to_hir_within_context(context)?.into(),
        })
    }
}

impl ASTLoweringWithinContext for ast::Assignment {
    type HIR = hir::Assignment;

    /// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
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
		context: &mut impl Context,
	) -> Result<Self::HIR, Error> {
		let value = self.value.as_ref().map(
			|expr| expr.lower_to_hir_within_context(context)
		).transpose()?;

		if let Some(f) = context.function() {
			let return_type = f.return_type();
			if let Some(value) = &value {
				if value.ty() != return_type {
					return Err(ReturnTypeMismatch {
						got: value.ty(),
						got_span: value.range().into(),

						expected: return_type,
					}.into());
				}
			}
			else if !return_type.is_none() {
				return Err(MissingReturnValue {
					ty: return_type,
					at: self.range().end.into(),
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
			context: &mut impl Context,
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
		context: &mut impl Context,
	) -> Result<Self::HIR, Error> {
		Ok(hir::Loop {
			body: self.body.iter().map(
				|stmt| stmt.lower_to_hir_within_context(context)
			).try_collect()?,
		})
	}
}

impl ASTLoweringWithinContext for ast::While {
	type HIR = hir::While;

	/// Lower [`ast::While`] to [`hir::While`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut impl Context,
		) -> Result<Self::HIR, Error> {
		Ok(hir::While {
			condition: self.condition.lower_condition_to_hir(context)?,
			body: self.body.iter().map(
				|stmt| stmt.lower_to_hir_within_context(context)
			).try_collect()?,
		})
	}
}

impl ASTLoweringWithinModule for ast::Module {
	type HIR = ();

	/// Lower [`ast::Module`] to [`hir::Module`] within lowering context
	fn lower_to_hir_within_context(
			&self,
			context: &mut ModuleContext,
		) -> Result<Self::HIR, Error> {
		let types = self.statements.iter().filter_map(
			|stmt|
			if let ast::Statement::Declaration(
				ast::Declaration::Type(_) | ast::Declaration::Trait(_)
			) = stmt {
				Some(stmt.lower_to_hir_within_context(context))
			}
			else {
				None
			}
		).try_collect::<Vec<_>>()?;

		for stmt in &self.statements {
			if let ast::Statement::Declaration(ast::Declaration::Function(f)) = stmt {
				f.declare(context)?;
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

		Ok(())
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
        let mut context = ModuleContext::default();
        self.lower_to_hir_within_context(&mut context)
    }
}

impl ASTLowering for ast::Module {
	type HIR = hir::Module;

	fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
		let mut context = ModuleContext::default();
		self.lower_to_hir_within_context(&mut context)?;
		Ok(context.module)
	}
}