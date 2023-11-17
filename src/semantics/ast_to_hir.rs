use core::panic;
use std::collections::HashMap;
use std::sync::Arc;

use crate::compilation::Compiler;
use crate::from_decimal::FromDecimal;
use crate::hir::{
    self, FunctionNamePart, Generic, GenericName, GenericType, Specialize, Type, TypeReference,
    Typed,
};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;
use crate::{AddSourceLocation, ErrVec, SourceLocation, WithSourceLocation};

use super::{error::*, Context, Declare, FindDeclaration, ModuleContext, MonomorphizedWithArgs};
use crate::ast::{self, CallNamePart, FnKind, If};

/// Lower AST inside some context
pub trait ASTLowering {
    type Error = Error;
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error>;

    /// Lower AST to HIR
    fn lower_to_hir(&self) -> Result<Self::HIR, Self::Error> {
        let mut compiler = Compiler::new();
        let mut context = ModuleContext::new(&mut compiler);
        self.lower_to_hir_within_context(&mut context)
    }
}

impl ASTLowering for ast::Statement {
    type HIR = hir::Statement;

    /// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(match self {
            ast::Statement::Declaration(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Assignment(assign) => {
                assign.lower_to_hir_within_context(context)?.into()
            }
            ast::Statement::Expression(expr) => expr.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Return(ret) => ret.lower_to_hir_within_context(context)?.into(),
            ast::Statement::If(stmt) => stmt.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Loop(stmt) => stmt.lower_to_hir_within_context(context)?.into(),
            ast::Statement::While(stmt) => stmt.lower_to_hir_within_context(context)?.into(),
            ast::Statement::Use(u) => u.lower_to_hir_within_context(context)?.into(),
        })
    }
}

impl ASTLowering for ast::Literal {
    type HIR = hir::Literal;

    /// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(match self {
            ast::Literal::None { offset } => hir::Literal::None {
                offset: *offset,
                ty: context.builtin().types().none(),
            },
            ast::Literal::Bool { offset, value } => hir::Literal::Bool {
                offset: *offset,
                value: *value,
                ty: context.builtin().types().bool(),
            },
            ast::Literal::Integer { value, .. } => hir::Literal::Integer {
                span: self.range(),
                value: value.parse::<rug::Integer>().unwrap(),
                ty: context.builtin().types().integer(),
            },
            ast::Literal::Rational { value, .. } => hir::Literal::Rational {
                span: self.range(),
                value: rug::Rational::from_decimal(&value).unwrap(),
                ty: context.builtin().types().rational(),
            },
            ast::Literal::String { value, .. } => hir::Literal::String {
                span: self.range(),
                value: value.clone(),
                ty: context.builtin().types().string(),
            },
        })
    }
}

impl ASTLowering for ast::VariableReference {
    type HIR = hir::VariableReference;

    /// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
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

/// Trait to convert one type to another
pub trait Convert {
    /// Convert this type to another type
    fn convert_to(&self, ty: WithSourceLocation<hir::Type>) -> ConversionRequest;
}

impl Convert for WithSourceLocation<hir::Type> {
    fn convert_to(&self, to: WithSourceLocation<hir::Type>) -> ConversionRequest {
        ConversionRequest {
            from: self.clone(),
            to,
        }
    }
}

/// Helper struct to perform check within context
pub struct ConversionRequest {
    from: WithSourceLocation<Type>,
    to: WithSourceLocation<Type>,
}

impl ConversionRequest {
    /// Convert one type to another within context
    pub fn within(&self, context: &impl FindDeclaration) -> Result<Type, NotConvertible> {
        let from = self.from.value.without_ref();
        let to = self.to.value.without_ref();
        let convertible = match (from.clone(), to.clone()) {
            (Type::Trait(tr), Type::SelfType(s)) => {
                Arc::ptr_eq(&tr, &s.associated_trait.upgrade().unwrap())
            }
            (Type::Class(c), Type::SelfType(s)) => c
                .at(self.from.source_location.clone())
                .implements(
                    s.associated_trait
                        .upgrade()
                        .unwrap()
                        .at(self.to.source_location.clone()),
                )
                .within(context)
                .map(|_| true)?,
            (Type::Class(c), Type::Trait(tr)) => c
                .at(self.from.source_location.clone())
                .implements(tr.clone().at(self.to.source_location.clone()))
                .within(context)
                .map(|_| true)?,
            (_, Type::Generic(to)) => {
                if let Some(constraint) = to.constraint {
                    self.from
                        .convert_to(
                            constraint
                                .referenced_type
                                .clone()
                                .at(self.to.source_location.clone()),
                        )
                        .within(context)
                        .map(|_| true)?
                } else {
                    true
                }
            }
            (Type::Generic(from), _) if from.constraint.is_some() => from
                .constraint
                .unwrap()
                .referenced_type
                .at(self.from.source_location.clone())
                .convert_to(self.to.clone())
                .within(context)
                .map(|_| true)?,
            (Type::Class(from), Type::Class(to)) => {
                if to.specialization_of == Some(from.clone())
                    || from.specialization_of.is_some()
                        && to.specialization_of == from.specialization_of
                {
                    from.generics()
                        .iter()
                        .zip(to.generics().iter())
                        .all(|(from, to)| {
                            from.clone()
                                .at(self.from.source_location.clone())
                                .convert_to(to.clone().at(self.to.source_location.clone()))
                                .within(context)
                                // TODO: Add error
                                .is_ok()
                        })
                } else {
                    from == to
                }
            }
            (from, to) => from == to,
        };

        if !convertible {
            return Err(TypeMismatch {
                // TODO: use WithSourceLocation for TypeWithSpan
                got: TypeWithSpan {
                    ty: self.from.value.clone(),
                    at: self.from.source_location.at.clone(),
                    source_file: self.from.source_location.source_file.clone(),
                },
                expected: TypeWithSpan {
                    ty: self.to.value.clone(),
                    at: self.to.source_location.at.clone(),
                    source_file: self.to.source_location.source_file.clone(),
                },
            }
            .into());
        }

        Ok(to)
    }
}

/// Trait to check if type implements trait
pub trait Implements {
    /// Does this class implement given trait?
    fn implements(&self, tr: WithSourceLocation<Arc<hir::TraitDeclaration>>) -> ImplementsCheck;
}

impl Implements for WithSourceLocation<Arc<hir::TypeDeclaration>> {
    fn implements(&self, tr: WithSourceLocation<Arc<hir::TraitDeclaration>>) -> ImplementsCheck {
        ImplementsCheck {
            ty: self.clone(),
            tr,
        }
    }
}

/// Helper struct to do check within context
pub struct ImplementsCheck {
    ty: WithSourceLocation<Arc<hir::TypeDeclaration>>,
    tr: WithSourceLocation<Arc<hir::TraitDeclaration>>,
}

impl ImplementsCheck {
    pub fn within(&self, context: &impl FindDeclaration) -> Result<(), NotImplemented> {
        let unimplemented: Vec<_> = self
            .tr
            .value
            .functions
            .values()
            .filter(|f| {
                matches!(f, hir::Function::Declaration(_))
                    && context
                        .find_implementation(&f, &Type::from(self.ty.value.clone()))
                        .is_none()
            })
            .cloned()
            .collect();

        if !unimplemented.is_empty() {
            return Err(NotImplemented {
                ty: self.ty.value.clone().into(),
                tr: self.tr.value.clone(),
                unimplemented,
            });
        }

        Ok(())
    }
}

impl ASTLowering for ast::Call {
    type HIR = hir::Call;

    /// Lower [`ast::Call`] to [`hir::Call`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        let args_cache: Vec<Option<hir::Expression>> = self
            .name_parts
            .iter()
            .map(|part| match part {
                CallNamePart::Argument(a) => Ok::<_, Error>(Some(
                    a.lower_to_hir_within_context(context)?
                )),
                CallNamePart::Text(t) => {
                    if let Some(var) = context.find_variable(t) {
                        return Ok(Some(
                            hir::VariableReference {
                                span: t.range().into(),
                                variable: var,
                            }
                            .into(),
                        ));
                    } else if t.as_str().chars().nth(0).is_some_and(|c| c.is_uppercase()) && let Some(ty) = context.find_type(t) {
                        return Ok(Some(
                            hir::TypeReference {
                                span: t.range().into(),
                                referenced_type: ty.clone(),
                                type_for_type: context.builtin().types().type_of(ty)
                            }.into()
                        ));
                    }
                    Ok(None)
                }
            })
            .try_collect()?;

        let args_cache: Vec<_> = args_cache
            .into_iter()
            .map(|e| {
                if let Some(hir::Expression::TypeReference(ty)) = e {
                    Some(ty.replace_with_type_info(context).into())
                } else {
                    e
                }
            })
            .collect();

        let candidates = context.candidates(&self.name_parts, &args_cache);

        let mut candidates_not_viable = Vec::new();
        for f in candidates {
            let mut modules = context
                .compiler()
                .modules
                .values()
                .map(|m| m.as_ref())
                .collect::<Vec<_>>();
            modules.push(context.module());

            let source_file = modules
                .iter()
                .find(|m| {
                    m.iter_functions()
                        .find(|function| function.name() == f.name())
                        .is_some()
                })
                .map(|m| m.source_file())
                .cloned();

            let mut args = Vec::new();
            let mut args_types = Vec::new();
            let mut failed = false;
            for (i, f_part) in f.name_parts().iter().enumerate() {
                match f_part {
                    FunctionNamePart::Text(_) => continue,
                    FunctionNamePart::Parameter(p) => {
                        let arg = args_cache[i].as_ref().unwrap();

                        let conversion = arg
                            .ty()
                            .at(arg.range())
                            .convert_to(p.ty().at(SourceLocation {
                                at: p.name.range().into(),
                                source_file: source_file.clone().map(Into::into),
                            }))
                            .within(context);
                        match conversion {
                            Ok(ty) => {
                                args_types.push(if ty.is_generic() { arg.ty() } else { ty });
                                args.push(arg.clone());
                            }
                            Err(err) => {
                                candidates_not_viable
                                    .push(CandidateNotViable { reason: err.into() });
                                failed = true;
                                break;
                            }
                        }
                    }
                }
            }

            if !failed {
                let function = f.monomorphized(context, args_types);
                let generic = if f.is_generic() { Some(f) } else { None };
                return Ok(hir::Call {
                    range: self.range(),
                    function,
                    generic,
                    args,
                });
            }
        }

        let arguments = args_cache
            .iter()
            .zip(&self.name_parts)
            .filter_map(|(arg, part)| {
                if matches!(part, CallNamePart::Argument(_)) {
                    let arg = arg.as_ref().unwrap();
                    Some((arg.ty(), arg.range().into()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut name = self.name_format().to_string();
        for arg in &arguments {
            name = name.replacen("<>", format!("<:{}>", arg.0).as_str(), 1);
        }

        let at = if self.kind == FnKind::Function {
            self.range()
        } else if matches!(self.name_parts[0], CallNamePart::Text(_)) {
            self.name_parts[0].range()
        } else {
            self.name_parts[1].range()
        };

        Err(NoFunction {
            kind: self.kind,
            name,
            arguments,
            candidates: candidates_not_viable,
            at: at.into(),
        }
        .into())
    }
}

impl ASTLowering for ast::Tuple {
    type HIR = hir::Expression;

    /// Lower [`ast::Tuple`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        if self.expressions.len() == 1 {
            return self.expressions[0].lower_to_hir_within_context(context);
        }
        todo!("real tuples")
    }
}

impl ASTLowering for ast::TypeReference {
    type HIR = hir::TypeReference;

    /// Lower [`ast::TypeReference`] to [`hir::TypeReference`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        let ty = context.find_type(&self.name);
        if ty.is_none() {
            return Err(UnknownType {
                name: self.name.clone().into(),
                at: self.name.range().into(),
            }
            .into());
        }
        let ty = ty.unwrap();

        let generics: Vec<_> = self
            .generic_parameters
            .iter()
            .map(|p| p.lower_to_hir_within_context(context))
            .try_collect()?;

        let generics_mapping = HashMap::from_iter(
            ty.generics()
                .into_iter()
                .cloned()
                .zip(generics.into_iter().map(|g| g.referenced_type)),
        );

        let ty = ty.specialize_with(&generics_mapping);

        let type_for_type = context.builtin().types().type_of(ty.clone());
        Ok(hir::TypeReference {
            span: self.range().into(),
            referenced_type: ty,
            type_for_type,
        })
    }
}

impl ASTLowering for ast::MemberReference {
    type HIR = hir::MemberReference;

    /// Lower [`ast::MemberReference`] to [`hir::MemberReference`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        let base = self.base.lower_to_hir_within_context(context)?;
        if let Some((index, member)) = base
            .ty()
            .members()
            .iter()
            .enumerate()
            .find(|(_, m)| m.name() == self.name.as_str())
        {
            Ok(hir::MemberReference {
                span: self.range().into(),
                base: Box::new(base),
                member: member.clone(),
                index,
            })
        } else {
            Err(NoMember {
                name: self.name.clone().into(),
                at: self.name.range().into(),
                ty: base.ty(),
                base_span: base.range().into(),
            }
            .into())
        }
    }
}

impl ASTLowering for ast::Constructor {
    type HIR = hir::Constructor;

    /// Lower [`ast::Constructor`] to [`hir::Constructor`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        let mut ty = self.ty.lower_to_hir_within_context(context)?;
        let generic_ty: Arc<hir::TypeDeclaration> =
            ty.referenced_type
                .clone()
                .try_into()
                .map_err(|_| NonClassConstructor {
                    ty: TypeWithSpan {
                        at: self.ty.range().into(),
                        ty: ty.referenced_type.clone(),
                        // TODO: real source file
                        source_file: None,
                    },
                })?;

        let mut members = ty.referenced_type.members().to_vec();

        let mut generics_map: HashMap<Type, Type> = HashMap::new();

        let mut initializers = Vec::<hir::Initializer>::new();
        for init in &self.initializers {
            let name = init.name.clone().unwrap_or_else(|| match &init.value {
                ast::Expression::VariableReference(var) => var.name.clone(),
                _ => unreachable!(),
            });
            let value = init.value.lower_to_hir_within_context(context)?;

            if let Some((index, member)) = ty
                .referenced_type
                .members()
                .iter()
                .enumerate()
                .find(|(_, m)| m.name() == name.as_str())
            {
                // FIXME: find in which file member type was declared
                value
                    .ty()
                    .at(value.range())
                    .convert_to(member.ty().at(member.name.range()))
                    .within(context)?;

                if let Some(prev) = initializers.iter().find(|i| i.index == index) {
                    return Err(MultipleInitialization {
                        name: member.name().to_string(),
                        at: [prev.range().into(), init.range().into()].into(),
                    }
                    .into());
                }

                if member.is_generic() {
                    // TODO: check for constraints
                    let diff = members[index].ty().diff(value.ty());
                    generics_map.extend(diff);
                    members[index] = Arc::new(hir::Member {
                        ty: members[index].ty().specialize_with(&generics_map),
                        ..members[index].as_ref().clone()
                    })
                }

                initializers.push(hir::Initializer {
                    span: name.range(),
                    index,
                    member: members[index].clone(),
                    value,
                });
            } else {
                return Err(NoMember {
                    name: name.clone().into(),
                    at: name.range().into(),
                    ty: ty.referenced_type.clone(),
                    base_span: self.ty.range().into(),
                }
                .into());
            }
        }

        if initializers.len() != generic_ty.members.len() {
            assert!(
                initializers.len() < generic_ty.members.len(),
                "impossible to have more initializers at this point"
            );
            let diff = (0..generic_ty.members.len())
                .filter(|i| initializers.iter().find(|init| init.index == *i).is_none());
            return Err(MissingFields {
                ty: ty.referenced_type.clone(),
                at: self.ty.name.range().into(),
                fields: diff
                    .map(|i| generic_ty.members[i].name().to_string())
                    .collect::<Vec<_>>()
                    .into(),
            }
            .into());
        }

        if generic_ty.is_generic() {
            ty.referenced_type = generic_ty.specialize_with(&generics_map).into();
        }
        Ok(hir::Constructor {
            ty,
            initializers,
            rbrace: self.rbrace,
        })
    }
}

impl ASTLowering for ast::Expression {
    type HIR = hir::Expression;

    /// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(match self {
            ast::Expression::Literal(l) => l.lower_to_hir_within_context(context)?.into(),
            ast::Expression::VariableReference(var) => {
                var.lower_to_hir_within_context(context)?.into()
            }
            ast::Expression::Call(call) => call.lower_to_hir_within_context(context)?.into(),
            ast::Expression::Tuple(t) => t.lower_to_hir_within_context(context)?.into(),
            ast::Expression::TypeReference(t) => t
                .lower_to_hir_within_context(context)?
                .replace_with_type_info(context)
                .into(),
            ast::Expression::MemberReference(m) => m.lower_to_hir_within_context(context)?.into(),
            ast::Expression::Constructor(c) => c.lower_to_hir_within_context(context)?.into(),
        })
    }
}

/// Trait for lowering conditional expression
trait Condition {
    /// Lower expression that is a condition
    fn lower_condition_to_hir(&self, context: &mut impl Context) -> Result<hir::Expression, Error>;
}

impl Condition for ast::Expression {
    fn lower_condition_to_hir(&self, context: &mut impl Context) -> Result<hir::Expression, Error> {
        let condition = self.lower_to_hir_within_context(context)?;
        if !condition.ty().is_bool() {
            return Err(ConditionTypeMismatch {
                got: condition.ty(),
                at: condition.range().into(),
            }
            .into());
        }

        Ok(condition)
    }
}

impl ASTLowering for ast::Member {
    type HIR = Arc<hir::Member>;

    /// Lower [`ast::Member`] to [`hir::Member`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(Arc::new(hir::Member {
            name: self.name.clone(),
            ty: self
                .ty
                .lower_to_hir_within_context(context)?
                .referenced_type,
        }))
    }
}

impl ASTLowering for ast::Parameter {
    type HIR = Arc<hir::Parameter>;

    /// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(Arc::new(hir::Parameter {
            name: self.name.clone(),
            ty: self
                .ty
                .lower_to_hir_within_context(context)?
                .referenced_type,
        }))
    }
}

impl ASTLowering for ast::Annotation {
    type HIR = hir::Annotation;

    /// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        _context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        // TODO: define annotations in code
        match self.name.as_str() {
            "mangle_as" => {
                if let Some(ast::Expression::Literal(ast::Literal::String { value, .. })) =
                    self.args.first()
                {
                    return Ok(hir::Annotation::MangleAs(value.clone()));
                }
            }
            "builtin" if self.args.is_empty() => return Ok(hir::Annotation::Builtin),
            _ => {}
        }
        Err(UnknownAnnotation {
            name: self.name.to_string(),
            at: self.name.range().into(),
        }
        .into())
    }
}

impl ASTLowering for ast::Assignment {
    type HIR = hir::Assignment;

    /// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
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
                got: TypeWithSpan {
                    ty: value.ty(),
                    at: self.value.range().into(),
                    source_file: None,
                },

                expected: TypeWithSpan {
                    ty: target.ty(),
                    at: self.target.range().into(),
                    // FIXME: find where variable was declared
                    source_file: None,
                },
            }
            .into());
        }

        Ok(hir::Assignment { target, value })
    }
}

impl ASTLowering for ast::Return {
    type HIR = hir::Return;

    /// Lower [`ast::Return`] to [`hir::Return`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        let value = self
            .value
            .as_ref()
            .map(|expr| expr.lower_to_hir_within_context(context))
            .transpose()?;

        if let Some(f) = context.function() {
            let return_type = f.return_type.clone();
            if let Some(value) = &value {
                if value.ty() != return_type {
                    return Err(ReturnTypeMismatch {
                        got: value.ty(),
                        got_span: value.range().into(),

                        expected: return_type,
                    }
                    .into());
                }
            } else if !return_type.is_none() {
                return Err(MissingReturnValue {
                    ty: return_type,
                    at: self.range().end.into(),
                }
                .into());
            }
        } else {
            return Err(ReturnOutsideFunction {
                at: self.range().into(),
            }
            .into());
        }

        Ok(hir::Return { value })
    }
}

impl ASTLowering for If {
    type HIR = hir::If;

    /// Lower [`ast::If`] to [`hir::If`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(hir::If {
            condition: self.condition.lower_condition_to_hir(context)?,
            body: self
                .body
                .iter()
                .map(|stmt| stmt.lower_to_hir_within_context(context))
                .try_collect()?,
            else_ifs: self
                .else_ifs
                .iter()
                .map(|else_if| {
                    Ok::<hir::ElseIf, Error>(hir::ElseIf {
                        condition: else_if.condition.lower_condition_to_hir(context)?,
                        body: else_if
                            .body
                            .iter()
                            .map(|stmt| stmt.lower_to_hir_within_context(context))
                            .try_collect()?,
                    })
                })
                .try_collect()?,
            else_block: self
                .else_block
                .iter()
                .map(|stmt| stmt.lower_to_hir_within_context(context))
                .try_collect()?,
        })
    }
}

impl ASTLowering for ast::Loop {
    type HIR = hir::Loop;

    /// Lower [`ast::Loop`] to [`hir::Loop`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(hir::Loop {
            body: self
                .body
                .iter()
                .map(|stmt| stmt.lower_to_hir_within_context(context))
                .try_collect()?,
        })
    }
}

impl ASTLowering for ast::While {
    type HIR = hir::While;

    /// Lower [`ast::While`] to [`hir::While`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(hir::While {
            condition: self.condition.lower_condition_to_hir(context)?,
            body: self
                .body
                .iter()
                .map(|stmt| stmt.lower_to_hir_within_context(context))
                .try_collect()?,
        })
    }
}

impl ASTLowering for ast::Use {
    type HIR = hir::Use;

    /// Lower [`ast::Use`] to [`hir::Use`] within lowering context
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        if self.path.len() != 2 {
            panic!("Currently only module.declaration_name usage is supported");
        }

        let module_name = self.path.first().unwrap().as_str();
        let module = context.compiler_mut().get_module(module_name).unwrap();

        let name = self.path.last().unwrap().as_str();

        let imported_item: hir::ImportedItem = if let Some(var) = module.variables.get(name) {
            context
                .module_mut()
                .variables
                .insert(var.name().to_string(), var.clone());
            var.clone().into()
        } else if let Some(ty) = module.types.get(name) {
            context
                .module_mut()
                .types
                .insert(ty.name().to_string(), ty.clone());
            ty.clone().into()
        } else if let Some(f) = module.iter_functions().find(|f| f.name() == name) {
            context.module_mut().insert_function(f.clone());
            f.clone().into()
        } else {
            todo!("Can't resolve imported name")
        };

        Ok(hir::Use {
            path: self.path.clone(),
            imported_item,
        })
    }
}

impl ASTLowering for ast::Module {
    type HIR = hir::Module;
    type Error = ErrVec<Error>;

    /// Lower [`ast::Module`] to [`hir::Module`] within lowering context
    ///
    /// # Order
    ///
    /// Lowering happens in 2 passes: `declaration`` and `definition`
    ///
    /// 1. Use statements
    /// 2. Types
    /// 3. Traits
    /// 4. Functions & Global variables (in the order they are declared)
    /// 5. Rest of statements
    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        use ast::Declaration as D;
        use ast::Statement as S;

        let mut errors = Vec::new();

        // Import things first
        self.statements
            .iter()
            .filter(|s| matches!(s, ast::Statement::Use(_)))
            .for_each(|stmt: &S| {
                let res = stmt.lower_to_hir_within_context(context);
                match res {
                    Ok(stmt) => context.module_mut().statements.push(stmt),
                    Err(err) => errors.push(err),
                }
            });

        for_each_declaration_in_order(&self.statements, |stmt: &S| {
            let decl: &D = match stmt {
                S::Declaration(d) => d,
                _ => return,
            };

            let res = decl.declare(context);
            if let Err(err) = res {
                errors.push(err);
            }
        });

        // TODO: find a way to reuse this above
        let mut define = |stmt: &S| {
            let res = stmt.lower_to_hir_within_context(context);
            match res {
                Ok(stmt) => context.module_mut().statements.push(stmt),
                Err(err) => errors.push(err),
            }
        };

        for_each_declaration_in_order(&self.statements, &mut define);

        // Add rest of statements
        self.statements
            .iter()
            .filter(|s| !matches!(s, S::Use(_) | S::Declaration(_)))
            .for_each(&mut define);

        if !errors.is_empty() {
            return Err(errors.into());
        }

        Ok(context.module().clone())
    }
}

/// Run some function for each decl in order:
/// 1. Types
/// 2. Traits
/// 3. Function & Global variables
fn for_each_declaration_in_order(stmts: &[ast::Statement], mut f: impl FnMut(&ast::Statement)) {
    use ast::Declaration as D;
    use ast::Statement as S;

    // First for types
    stmts
        .iter()
        .filter(|s| matches!(s, S::Declaration(D::Type(_))))
        .for_each(&mut f);

    // Then for traits
    stmts
        .iter()
        .filter(|s| matches!(s, S::Declaration(D::Trait(_))))
        .for_each(&mut f);

    // Then for functions & global variables
    stmts
        .iter()
        .filter(|s| matches!(s, S::Declaration(D::Function(_) | D::Variable(_))))
        .for_each(&mut f);
}

/// Trait to replace [`TypeReference`] with type info
pub trait ReplaceWithTypeInfo {
    /// Replace [`TypeReference`] with type info
    fn replace_with_type_info(&self, context: &impl Context) -> hir::Expression;
}

impl ReplaceWithTypeInfo for TypeReference {
    fn replace_with_type_info(&self, context: &impl Context) -> hir::Expression {
        hir::Constructor {
            ty: hir::TypeReference {
                span: self.range(),
                referenced_type: self.type_for_type.clone(),
                type_for_type: context
                    .builtin()
                    .types()
                    .type_of(self.type_for_type.clone()),
            },
            initializers: vec![
                hir::Initializer {
                    span: 0..0,
                    index: 0,
                    member: self.type_for_type.members()[0].clone(),
                    value: hir::Literal::String {
                        span: 0..0,
                        value: self.referenced_type.generic_name().to_string(),
                        ty: context.builtin().types().string(),
                    }
                    .into(),
                },
                hir::Initializer {
                    span: 0..0,
                    index: 1,
                    member: self.type_for_type.members()[1].clone(),
                    value: hir::Literal::Integer {
                        span: 0..0,
                        value: self.referenced_type.size_in_bytes().into(),
                        ty: context.builtin().types().integer(),
                    }
                    .into(),
                },
            ],
            rbrace: self.end() - 1,
        }
        .into()
    }
}

impl ASTLowering for ast::GenericParameter {
    type HIR = Type;
    type Error = Error;

    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        Ok(GenericType {
            name: self.name.clone(),
            constraint: self
                .constraint
                .as_ref()
                .map(|ty| ty.lower_to_hir_within_context(context))
                .transpose()?,
        }
        .into())
    }
}

impl<T: ASTLowering> ASTLowering for Vec<T> {
    type HIR = Vec<T::HIR>;
    type Error = T::Error;

    fn lower_to_hir_within_context(
        &self,
        context: &mut impl Context,
    ) -> Result<Self::HIR, Self::Error> {
        self.iter()
            .map(|t| t.lower_to_hir_within_context(context))
            .try_collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_compilation_result;

    test_compilation_result!(candidate_not_viable);
    test_compilation_result!(constraints);
    test_compilation_result!(generics);
    test_compilation_result!(missing_fields);
    test_compilation_result!(multiple_errors);
    test_compilation_result!(multiple_initialization);
    test_compilation_result!(non_class_constructor);
    test_compilation_result!(predeclare_vars);
    test_compilation_result!(references);
    test_compilation_result!(traits);
    test_compilation_result!(type_as_value);
    test_compilation_result!(wrong_initializer_type);
}
