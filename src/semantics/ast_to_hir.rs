use core::panic;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::compilation::Compiler;
use crate::from_decimal::FromDecimal;
use crate::hir::{self, FunctionNamePart, GenericType, Type, Typed};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;

use super::{error::*, Context, Declare, GenericContext, ModuleContext, MonomorphizedWithArgs};
use crate::ast::{self, CallNamePart, FnKind, If};

/// Lower AST inside some context
pub trait ASTLoweringWithinContext {
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error>;
}

pub trait ASTLoweringWithinModule {
    type HIR;

    /// Lower AST to HIR within some context
    fn lower_to_hir_within_context(&self, context: &mut ModuleContext) -> Result<Self::HIR, Error>;
}

impl ASTLoweringWithinContext for ast::Statement {
    type HIR = hir::Statement;

    /// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::Literal {
    type HIR = hir::Literal;

    /// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::VariableReference {
    type HIR = hir::VariableReference;

    /// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

/// Trait to check if one type is convertible to another within context
trait ConvertibleTo {
    /// Is this type convertible to another?
    fn convertible_to(&self, ty: hir::Type) -> ConvertibleToCheck;
}

impl ConvertibleTo for hir::Type {
    fn convertible_to(&self, ty: hir::Type) -> ConvertibleToCheck {
        ConvertibleToCheck {
            from: self.clone(),
            to: ty,
        }
    }
}

/// Helper struct to perform check within context
struct ConvertibleToCheck {
    from: Type,
    to: Type,
}

impl ConvertibleToCheck {
    pub fn within(&self, context: &impl Context) -> bool {
        match (&self.from, &self.to) {
            (Type::Trait(tr), Type::SelfType(s)) => {
                Arc::ptr_eq(&tr, &s.associated_trait.upgrade().unwrap())
            }
            // TODO: this needs context of all visible functions to check if class implements trait
            (Type::Class(c), Type::Trait(tr)) => c.implements(tr.clone()).within(context),
            // TODO: check for constraints
            (_, Type::Generic(_)) => true,
            _ => self.from == self.to,
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
            tr: tr,
        }
    }
}

/// Helper struct to do check within context
struct ImplementsCheck {
    ty: Type,
    tr: Arc<hir::TraitDeclaration>,
}

impl ImplementsCheck {
    pub fn within(&self, context: &impl Context) -> bool {
        self.tr
            .functions
            .iter()
            .all(|f| context.find_implementation(&f, &self.ty).is_some())
    }
}

impl ASTLoweringWithinContext for ast::Call {
    type HIR = hir::Call;

    /// Lower [`ast::Call`] to [`hir::Call`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        let args_cache = self
            .name_parts
            .iter()
            .map(|part| match part {
                CallNamePart::Argument(a) => Ok::<Option<hir::Expression>, Error>(Some(
                    a.lower_to_hir_within_context(context)?,
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
                    }
                    Ok(None)
                }
            })
            .try_collect::<Vec<_>>()?;

        let candidates = context.candidates(&self.name_parts, &args_cache);

        let mut candidates_not_viable = Vec::new();
        for f in candidates {
            let builtin = context.is_for_builtin_module();
            // FIXME: compiler should have builtin module too
            let mut modules = context
                .compiler()
                .modules
                .values()
                .map(|m| m.as_ref())
                .collect::<Vec<_>>();
            if !builtin {
                modules.push(hir::Module::builtin());
            }

            let source_code = modules
                .iter()
                .find(|m| {
                    m.iter_functions()
                        .find(|function| function.name() == f.name())
                        .is_some()
                })
                .map(|m| fs::read_to_string(Path::new(&m.filename)).ok())
                .flatten();

            let mut args = Vec::new();
            let mut failed = false;
            for (i, f_part) in f.name_parts().iter().enumerate() {
                match f_part {
                    FunctionNamePart::Text(_) => continue,
                    FunctionNamePart::Parameter(p) => {
                        let arg = args_cache[i].as_ref().unwrap();
                        if !arg.ty().convertible_to(p.ty()).within(context) {
                            candidates_not_viable.push(CandidateNotViable {
                                reason: TypeMismatch {
                                    expected: TypeWithSpan {
                                        ty: p.ty(),
                                        at: p.name.range().into(),
                                        source_code,
                                    },
                                    got: TypeWithSpan {
                                        ty: arg.ty(),
                                        at: arg.range().into(),
                                        source_code: None,
                                    },
                                }
                                .into(),
                            });
                            failed = true;
                            break;
                        }
                        args.push(arg.clone());
                    }
                }
            }

            if !failed {
                let function = f.monomorphized(context, args.iter().map(|a| a.ty()));
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

impl ASTLoweringWithinContext for ast::Tuple {
    type HIR = hir::Expression;

    /// Lower [`ast::Tuple`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        if self.expressions.len() == 1 {
            return self.expressions[0].lower_to_hir_within_context(context);
        }
        todo!("real tuples")
    }
}

impl ASTLoweringWithinContext for ast::TypeReference {
    type HIR = hir::TypeReference;

    /// Lower [`ast::TypeReference`] to [`hir::TypeReference`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        let ty = context.find_type(&self.name);
        if ty.is_none() {
            return Err(UnknownType {
                name: self.name.clone().into(),
                at: self.name.range().into(),
            }
            .into());
        }
        Ok(hir::TypeReference {
            span: self.range().into(),
            referenced_type: ty.unwrap(),
        })
    }
}

impl ASTLoweringWithinContext for ast::MemberReference {
    type HIR = hir::MemberReference;

    /// Lower [`ast::MemberReference`] to [`hir::MemberReference`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::Constructor {
    type HIR = hir::Constructor;

    /// Lower [`ast::Constructor`] to [`hir::Constructor`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        let mut ty = self.ty.lower_to_hir_within_context(context)?;

        let mut members = ty.referenced_type.members().to_vec();

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
                if !value.ty().convertible_to(member.ty()).within(context) {
                    return Err(TypeMismatch {
                        expected: TypeWithSpan {
                            ty: member.ty(),
                            at: member.name.range().into(),
                            // FIXME: find in which file type was declared
                            source_code: None,
                        },
                        got: TypeWithSpan {
                            ty: value.ty(),
                            at: value.range().into(),
                            source_code: None,
                        },
                    }
                    .into());
                }

                if let Some(prev) = initializers.iter().find(|i| i.index == index) {
                    return Err(MultipleInitialization {
                        name: member.name().to_string(),
                        at: [prev.range().into(), init.range().into()].into(),
                    }
                    .into());
                }

                // TODO: move to monomorphize
                if member.ty.is_generic() {
                    members[index] = Arc::new(hir::Member {
                        name: member.name.clone(),
                        ty: value.ty().clone(),
                    });
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

        let generic_ty: Arc<hir::TypeDeclaration> = ty
            .referenced_type
            .clone()
            .try_into()
            .expect("constructors only meant for classes");

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

        ty.referenced_type = Arc::new(hir::TypeDeclaration {
            members,
            ..(*generic_ty).clone()
        })
        .into();

        Ok(hir::Constructor {
            ty,
            initializers,
            rbrace: self.rbrace,
        })
    }
}

impl ASTLoweringWithinContext for ast::Expression {
    type HIR = hir::Expression;

    /// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Expression::Literal(l) => l.lower_to_hir_within_context(context)?.into(),
            ast::Expression::VariableReference(var) => {
                var.lower_to_hir_within_context(context)?.into()
            }
            ast::Expression::Call(call) => call.lower_to_hir_within_context(context)?.into(),
            ast::Expression::Tuple(t) => t.lower_to_hir_within_context(context)?.into(),
            ast::Expression::TypeReference(t) => t.lower_to_hir_within_context(context)?.into(),
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

impl ASTLoweringWithinContext for ast::VariableDeclaration {
    type HIR = Arc<hir::VariableDeclaration>;

    /// Lower [`ast::VariableDeclaration`] to [`hir::VariableDeclaration`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        // TODO: check for collisions, etc
        let generic_parameters: Vec<_> = self
            .generic_parameters
            .iter()
            .cloned()
            .map(|name| GenericType { name })
            .collect();

        let is_builtin = context.is_for_builtin_module();
        let mut generic_context = GenericContext {
            parent: context,
            generic_parameters: generic_parameters.clone(),
        };

        // TODO: recursive types
        let ty = Arc::new(hir::TypeDeclaration {
            name: self.name.clone(),
            generic_parameters,
            is_builtin,
            members: self
                .members
                .iter()
                .map(|m| m.lower_to_hir_within_context(&mut generic_context))
                .try_collect()?,
        });

        context.add_type(ty.clone());

        Ok(ty)
    }
}

impl ASTLoweringWithinContext for ast::Member {
    type HIR = Arc<hir::Member>;

    /// Lower [`ast::Member`] to [`hir::Member`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        Ok(Arc::new(hir::Member {
            name: self.name.clone(),
            ty: self
                .ty
                .lower_to_hir_within_context(context)?
                .referenced_type,
        }))
    }
}

impl ASTLoweringWithinContext for ast::Parameter {
    type HIR = Arc<hir::Parameter>;

    /// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        Ok(Arc::new(hir::Parameter {
            name: self.name.clone(),
            ty: self
                .ty
                .lower_to_hir_within_context(context)?
                .referenced_type,
        }))
    }
}

impl ASTLoweringWithinContext for ast::Annotation {
    type HIR = hir::Annotation;

    /// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
    fn lower_to_hir_within_context(&self, _context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::FunctionDeclaration {
    type HIR = hir::Function;

    /// Lower [`ast::FunctionDeclaration`] to [`hir::Function`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        self.define(self.declare(context)?, context)
    }
}

impl ASTLoweringWithinContext for ast::TraitDeclaration {
    type HIR = Arc<hir::TraitDeclaration>;

    /// Lower [`ast::TraitDeclaration`] to [`hir::TraitDeclaration`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        self.define(self.declare(context)?, context)
    }
}

impl ASTLoweringWithinContext for ast::Declaration {
    type HIR = hir::Declaration;

    /// Lower [`ast::Declaration`] to [`hir::Declaration`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        Ok(match self {
            ast::Declaration::Variable(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Type(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Function(decl) => decl.lower_to_hir_within_context(context)?.into(),
            ast::Declaration::Trait(decl) => decl.lower_to_hir_within_context(context)?.into(),
        })
    }
}

impl ASTLoweringWithinContext for ast::Assignment {
    type HIR = hir::Assignment;

    /// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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
                    source_code: None,
                },

                expected: TypeWithSpan {
                    ty: target.ty(),
                    at: self.target.range().into(),
                    // FIXME: find where variable was declared
                    source_code: None,
                },
            }
            .into());
        }

        Ok(hir::Assignment { target, value })
    }
}

impl ASTLoweringWithinContext for ast::Return {
    type HIR = hir::Return;

    /// Lower [`ast::Return`] to [`hir::Return`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for If {
    type HIR = hir::If;

    /// Lower [`ast::If`] to [`hir::If`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::Loop {
    type HIR = hir::Loop;

    /// Lower [`ast::Loop`] to [`hir::Loop`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        Ok(hir::Loop {
            body: self
                .body
                .iter()
                .map(|stmt| stmt.lower_to_hir_within_context(context))
                .try_collect()?,
        })
    }
}

impl ASTLoweringWithinContext for ast::While {
    type HIR = hir::While;

    /// Lower [`ast::While`] to [`hir::While`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
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

impl ASTLoweringWithinContext for ast::Use {
    type HIR = hir::Use;

    /// Lower [`ast::Use`] to [`hir::Use`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        if self.path.len() != 2 {
            panic!("Currently only module.declaration_name usage is supported");
        }

        let module_name = self.path.first().unwrap().as_str();
        let module = context.compiler().get_module(module_name).unwrap();

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

impl ASTLoweringWithinModule for ast::Module {
    type HIR = ();

    /// Lower [`ast::Module`] to [`hir::Module`] within lowering context
    fn lower_to_hir_within_context(&self, context: &mut ModuleContext) -> Result<Self::HIR, Error> {
        // Import things first
        for stmt in &self.statements {
            if let ast::Statement::Use(u) = stmt {
                u.lower_to_hir_within_context(context)?;
            }
        }

        // Add types then
        for stmt in &self.statements {
            match stmt {
                ast::Statement::Declaration(ast::Declaration::Type(_)) => {
                    stmt.lower_to_hir_within_context(context)?;
                }
                ast::Statement::Declaration(ast::Declaration::Trait(tr)) => {
                    tr.declare(context)?;
                }
                _ => {}
            }
        }

        // Declare functions, but don't define them yet
        for stmt in &self.statements {
            if let ast::Statement::Declaration(ast::Declaration::Function(f)) = stmt {
                f.declare(context)?;
            }
        }

        // Add rest of statements
        for stmt in &self.statements {
            if matches!(
                stmt,
                ast::Statement::Use(_) | ast::Statement::Declaration(ast::Declaration::Type(_))
            ) {
                continue;
            }

            let stmt = stmt.lower_to_hir_within_context(context)?;
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
        let mut compiler = Compiler::new();
        let mut context = ModuleContext::new(&mut compiler);
        self.lower_to_hir_within_context(&mut context)
    }
}

impl ASTLowering for ast::Module {
    type HIR = hir::Module;

    fn lower_to_hir(&self) -> Result<Self::HIR, Error> {
        let mut compiler = Compiler::new();
        let mut context = ModuleContext::new(&mut compiler);
        self.lower_to_hir_within_context(&mut context)?;
        Ok(context.module)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_compiler_error;

    test_compiler_error!(candidate_not_viable);
    test_compiler_error!(missing_fields);
    test_compiler_error!(multiple_initialization);
    test_compiler_error!(wrong_initializer_type);
}
