use core::panic;
use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::compilation::Compiler;
use crate::from_decimal::FromDecimal;
use crate::hir::{
    self, FunctionNamePart, Generic, GenericType, ModuleData, Parameter, Specialize, Type,
    TypeReference, Typed, Variable, VariableData,
};
use crate::mutability::{Mutability, Mutable};
use crate::named::Named;
use crate::syntax::{Identifier, Keyword, Ranged};
use crate::{AddSourceLocation, ErrVec, SourceLocation, WithSourceLocation};

use super::{
    error::*, AddDeclaration, Context, Convert, ConvertibleTo, Declare, FindDeclaration,
    GenericContext, Implicit, ModuleContext,
};
use crate::ast::{self, CallNamePart, FnKind, If};
use crate::semantics::monomorphize::Monomorphize;

/// Lower to HIR inside some context
pub trait ToHIR {
    type Error = Error;
    type HIR;

    /// Lower to HIR within some context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error>;

    /// Lower to HIR without context
    fn to_hir_without_context(&self) -> Result<Self::HIR, Self::Error> {
        let mut compiler = Compiler::new();
        let mut context = ModuleContext::new(ModuleData::default(), &mut compiler);
        self.to_hir(&mut context)
    }
}

impl ToHIR for ast::Statement {
    type HIR = hir::Statement;

    /// Lower [`ast::Statement`] to [`hir::Statement`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(match self {
            ast::Statement::Declaration(decl) => decl.to_hir(context)?.into(),
            ast::Statement::Assignment(assign) => assign.to_hir(context)?.into(),
            ast::Statement::Expression(expr) => expr.to_hir(context)?.into(),
            ast::Statement::Return(ret) => ret.to_hir(context)?.into(),
            ast::Statement::If(stmt) => stmt.to_hir(context)?.into(),
            ast::Statement::Loop(stmt) => stmt.to_hir(context)?.into(),
            ast::Statement::While(stmt) => stmt.to_hir(context)?.into(),
            ast::Statement::Use(u) => u.to_hir(context)?.into(),
        })
    }
}

impl ToHIR for ast::Literal {
    type HIR = hir::Literal;

    /// Lower [`ast::Literal`] to [`hir::Literal`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
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

impl ToHIR for ast::VariableReference {
    type HIR = hir::VariableReference;

    /// Lower [`ast::VariableReference`] to [`hir::VariableReference`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let var = context.find_variable(&self.name);
        if var.is_none() {
            return Err(UndefinedVariable {
                name: self.name.clone().to_string(),
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

impl ToHIR for ast::Call {
    type HIR = hir::Call;

    /// Lower [`ast::Call`] to [`hir::Call`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let args_cache: Vec<Option<hir::Expression>> = self
            .name_parts
            .iter()
            .map(|part| match part {
                CallNamePart::Argument(a) => Ok::<_, Error>(Some(a.to_hir(context)?)),
                CallNamePart::Text(t) => {
                    if let Some(var) = context.find_variable(t) {
                        return Ok(Some(
                            hir::VariableReference {
                                span: t.range().into(),
                                variable: var,
                            }
                            .into(),
                        ));
                    } else if t.as_str().chars().nth(0).is_some_and(|c| c.is_uppercase())
                        && let Some(ty) = context.find_type(t)
                    {
                        return Ok(Some(
                            hir::TypeReference {
                                span: t.range().into(),
                                referenced_type: ty.clone(),
                                type_for_type: context.builtin().types().type_of(ty),
                            }
                            .into(),
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
            let source_file = f
                .read()
                .unwrap()
                .module
                .data(context.compiler())
                .source_file()
                .clone();

            let mut args = Vec::new();
            let mut failed = false;
            GenericContext::for_fn(f.clone(), context).run(|context| {
                for (i, f_part) in f.read().unwrap().name_parts().iter().enumerate() {
                    match f_part {
                        FunctionNamePart::Text(_) => continue,
                        FunctionNamePart::Parameter(p) => {
                            let arg = args_cache[i].as_ref().unwrap();

                            let arg_source_file = context.compiler().current_file().clone();

                            let arg = WithSourceLocation {
                                value: arg.clone(),
                                source_location: SourceLocation {
                                    source_file: Some(arg_source_file),
                                    at: arg.range().into(),
                                },
                            }
                            .convert_to(p.ty().at(SourceLocation {
                                at: p.read().unwrap().name.range().into(),
                                source_file: Some(source_file.clone()),
                            }))
                            .within(context);
                            match arg {
                                Ok(arg) => {
                                    args.push(arg);
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
            });

            if !failed {
                if f.read().unwrap().return_type == Type::Unknown {
                    // TODO: specify that we can't deduce because it's called to early
                    // TODO: try to reverse order in which we process function definitions
                    return Err(CantDeduceType {
                        at: self.range().into(),
                    }
                    .into());
                }

                let generic = if f.read().unwrap().is_generic() {
                    Some(f.clone())
                } else {
                    None
                };

                let mut call = hir::Call {
                    range: self.range(),
                    function: f,
                    generic,
                    args,
                };
                call.monomorphize(context);
                return Ok(call);
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

impl ToHIR for ast::Tuple {
    type HIR = hir::Expression;

    /// Lower [`ast::Tuple`] to [`hir::Expression`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        if self.expressions.len() == 1 {
            return self.expressions[0].to_hir(context);
        }
        todo!("real tuples")
    }
}

impl ToHIR for ast::TypeReference {
    type HIR = hir::TypeReference;

    /// Lower [`ast::TypeReference`] to [`hir::TypeReference`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let name = match self.name {
            ast::Typename::Identifier(ref name) => name.as_str(),
            ast::Typename::Reference { mutable, .. } if mutable.is_some() => "ReferenceMut",
            ast::Typename::Reference { .. } => "Reference",
        };

        let ty = context.find_type(name);
        if ty.is_none() {
            return Err(UnknownType {
                name: self.name.clone().to_string(),
                at: self.name.range().into(),
            }
            .into());
        }
        let ty = ty.unwrap();

        let generics: Vec<_> = self
            .generic_parameters
            .iter()
            .map(|p| p.to_hir(context))
            .try_collect()?;

        let generics_mapping = HashMap::from_iter(
            ty.generics()
                .into_iter()
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

impl ToHIR for ast::MemberReference {
    type HIR = hir::MemberReference;

    /// Lower [`ast::MemberReference`] to [`hir::MemberReference`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let base = self.base.to_hir(context)?;
        if let Some((index, member)) = base
            .ty()
            .without_ref()
            .members()
            .iter()
            .enumerate()
            .find(|(_, m)| m.name() == self.name.as_str())
        {
            let base = base.dereference();
            Ok(hir::MemberReference {
                span: self.range().into(),
                base: Box::new(base),
                member: member.clone(),
                index,
            })
        } else {
            Err(NoMember {
                name: self.name.clone().to_string(),
                at: self.name.range().into(),
                ty: base.ty(),
                base_span: base.range().into(),
            }
            .into())
        }
    }
}

impl ToHIR for ast::Constructor {
    type HIR = hir::Constructor;

    /// Lower [`ast::Constructor`] to [`hir::Constructor`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let mut ty = self.ty.to_hir(context)?;
        let generic_ty: hir::Class =
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

        let mut constructor_context = GenericContext {
            generic_parameters: generic_ty
                .read()
                .unwrap()
                .generics()
                .into_iter()
                .cloned()
                .collect(),
            generics_mapping: HashMap::new(),
            parent: context,
        };

        let mut initializers = Vec::<hir::Initializer>::new();
        for init in &self.initializers {
            let name = init.name.clone().unwrap_or_else(|| match &init.value {
                ast::Expression::VariableReference(var) => var.name.clone(),
                _ => unreachable!(),
            });
            let value = init.value.to_hir(&mut constructor_context)?;

            if let Some((index, member)) = ty
                .referenced_type
                .members()
                .iter()
                .enumerate()
                .find(|(_, m)| m.name() == name.as_str())
            {
                let value = WithSourceLocation {
                    value: value.clone(),
                    source_location: SourceLocation {
                        // FIXME: find in which file member type was declared
                        source_file: None,
                        at: value.range().into(),
                    },
                }
                .convert_to(member.ty().at(member.name.range()))
                .within(&mut constructor_context)?;

                if let Some(prev) = initializers.iter().find(|i| i.index == index) {
                    return Err(MultipleInitialization {
                        name: member.name().to_string(),
                        first_at: prev.range().into(),
                        repeated_at: vec![init.range().into()],
                    }
                    .into());
                }

                if member.is_generic() {
                    members[index] = Arc::new(hir::Member {
                        ty: value.ty(),
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
                    name: name.clone().to_string(),
                    at: name.range().into(),
                    ty: ty.referenced_type.clone(),
                    base_span: self.ty.range().into(),
                }
                .into());
            }
        }

        let len = generic_ty.read().unwrap().members().len();
        if initializers.len() != len {
            assert!(
                initializers.len() < len,
                "impossible to have more initializers at this point"
            );
            let diff =
                (0..len).filter(|i| initializers.iter().find(|init| init.index == *i).is_none());
            return Err(MissingFields {
                ty: ty.referenced_type.clone(),
                at: self.ty.name.range().into(),
                fields: diff
                    .map(|i| generic_ty.read().unwrap().members()[i].name().to_string())
                    .collect::<Vec<_>>()
                    .into(),
            }
            .into());
        }

        if generic_ty.is_generic() {
            ty.referenced_type = generic_ty
                .specialize_with(&constructor_context.generics_mapping)
                .into();
        }
        Ok(hir::Constructor {
            ty,
            initializers,
            rbrace: self.rbrace,
        })
    }
}

impl ToHIR for ast::Expression {
    type HIR = hir::Expression;

    /// Lower [`ast::Expression`] to [`hir::Expression`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(match self {
            ast::Expression::Literal(l) => l.to_hir(context)?.into(),
            ast::Expression::VariableReference(var) => var.to_hir(context)?.into(),
            ast::Expression::Call(call) => call.to_hir(context)?.into(),
            ast::Expression::Tuple(t) => t.to_hir(context)?.into(),
            ast::Expression::TypeReference(t) => {
                t.to_hir(context)?.replace_with_type_info(context).into()
            }
            ast::Expression::MemberReference(m) => m.to_hir(context)?.into(),
            ast::Expression::Constructor(c) => c.to_hir(context)?.into(),
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
        let condition = self.to_hir(context)?;
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

impl ToHIR for ast::Member {
    type HIR = Arc<hir::Member>;

    /// Lower [`ast::Member`] to [`hir::Member`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(Arc::new(hir::Member {
            name: self.name.clone(),
            ty: self.ty.to_hir(context)?.referenced_type,
        }))
    }
}

impl ToHIR for ast::Parameter {
    type HIR = hir::Parameter;

    /// Lower [`ast::Parameter`] to [`hir::Parameter`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let ty = self.ty.to_hir(context)?;
        let ty = if matches!(ty.referenced_type, Type::Trait(_)) {
            let span = ty.span.clone();
            let referenced_type: Type = context.new_generic_for_trait(ty).into();
            TypeReference {
                span,
                referenced_type: referenced_type.clone(),
                type_for_type: context.builtin().types().type_of(referenced_type),
            }
        } else {
            ty
        };
        Ok(Parameter::new(hir::ParameterData {
            name: self.name.clone(),
            ty,
            range: self.less..self.greater + 1,
        }))
    }
}

impl ToHIR for ast::Annotation {
    type HIR = hir::Annotation;

    /// Lower [`ast::Annotation`] to [`hir::Annotation`] within lowering context
    fn to_hir(&self, _context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
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

impl ToHIR for ast::Assignment {
    type HIR = hir::Assignment;

    /// Lower [`ast::Assignment`] to [`hir::Assignment`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let target = self.target.to_hir(context)?;
        if target.is_immutable() {
            return Err(AssignmentToImmutable {
                at: self.target.range().into(),
            }
            .into());
        }

        let value = self.value.to_hir(context)?;
        let value = value
            .convert_to(target.ty().without_ref().at(target.range()))
            .within(context)?;

        Ok(hir::Assignment { target, value })
    }
}

impl ToHIR for ast::Return {
    type HIR = hir::Return;

    /// Lower [`ast::Return`] to [`hir::Return`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let value = self
            .value
            .as_ref()
            .map(|expr| expr.to_hir(context))
            .transpose()?;

        if let Some(f) = context.function() {
            let return_type = f.read().unwrap().return_type.clone();
            if let Some(value) = &value {
                if !value
                    .ty()
                    .convertible_to(return_type.clone())
                    .within(context)
                    .is_ok_and(|convertible| convertible)
                {
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

        Ok(hir::Return::Explicit {
            keyword: self.keyword.clone(),
            value,
        })
    }
}

impl ToHIR for If {
    type HIR = hir::If;

    /// Lower [`ast::If`] to [`hir::If`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        let condition = self.condition.lower_condition_to_hir(context)?;
        let body = self
            .body
            .iter()
            .map(|stmt| stmt.to_hir(context))
            .try_collect()?;
        let else_ifs = self
            .else_ifs
            .iter()
            .map(|else_if| {
                Ok::<hir::ElseIf, Error>(hir::ElseIf {
                    else_keyword: else_if.else_keyword.clone(),
                    if_keyword: else_if.if_keyword.clone(),
                    condition: else_if.condition.lower_condition_to_hir(context)?,
                    body: else_if
                        .body
                        .iter()
                        .map(|stmt| stmt.to_hir(context))
                        .try_collect()?,
                })
            })
            .try_collect()?;
        let else_block = if let Some(else_block) = &self.else_block {
            Some(hir::Else {
                keyword: else_block.keyword.clone(),
                body: else_block
                    .body
                    .iter()
                    .map(|stmt| stmt.to_hir(context))
                    .try_collect()?,
            })
        } else {
            None
        };
        Ok(hir::If {
            keyword: self.keyword.clone(),
            condition,
            body,
            else_ifs,
            else_block,
        })
    }
}

impl ToHIR for ast::Loop {
    type HIR = hir::Loop;

    /// Lower [`ast::Loop`] to [`hir::Loop`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(hir::Loop {
            keyword: self.keyword.clone(),
            body: self
                .body
                .iter()
                .map(|stmt| stmt.to_hir(context))
                .try_collect()?,
        })
    }
}

impl ToHIR for ast::While {
    type HIR = hir::While;

    /// Lower [`ast::While`] to [`hir::While`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(hir::While {
            keyword: self.keyword.clone(),
            condition: self.condition.lower_condition_to_hir(context)?,
            body: self
                .body
                .iter()
                .map(|stmt| stmt.to_hir(context))
                .try_collect()?,
        })
    }
}

impl ToHIR for ast::Use {
    type HIR = hir::Use;

    /// Lower [`ast::Use`] to [`hir::Use`] within lowering context
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        if self.path.len() != 2 {
            panic!("Currently only module.declaration_name usage is supported");
        }

        let compiler = context.compiler_mut();

        let first_part = self.path.first().unwrap().as_str();
        let module = if compiler.locate(first_part).is_ok() {
            compiler.compile(first_part).unwrap()
        } else {
            let package = compiler.compile_package(first_part).unwrap();

            let current_package = compiler.current_package();
            current_package
                .data_mut(compiler)
                .dependencies
                .insert(package.clone());

            package
                .data(context.compiler())
                .modules
                .first()
                .unwrap()
                .clone()
        };

        let name = self.path.last().unwrap().as_str();

        let mut functions = IndexMap::new();
        let mut variables = IndexMap::new();
        let mut types = IndexMap::new();

        let module = module.data(context.compiler());
        let imported_item: hir::ImportedItem = if name == "*" {
            functions = module.functions.clone();
            variables = module.variables.clone();
            types = module.types.clone();
            hir::ImportedItem::All
        } else if let Some(var) = module.variables.get(name) {
            variables.insert(var.name().to_string(), var.clone());
            var.clone().into()
        } else if let Some(ty) = module.types.get(name) {
            types.insert(ty.name().to_string(), ty.clone());
            ty.clone().into()
        } else if let Some(f) = module.iter_functions().find(|f| f.name() == name) {
            let set = functions
                .entry(f.read().unwrap().name_format().to_string())
                .or_insert_with(IndexMap::new);
            set.insert(f.name().to_string(), f.clone().into());
            f.clone().into()
        } else {
            return Err::<_, Self::Error>(
                UnresolvedImport {
                    name: name.to_string(),
                    at: self.path.last().unwrap().range().into(),
                }
                .into(),
            );
        };

        context.module_mut().functions.extend(functions);
        context.module_mut().variables.extend(variables);
        context.module_mut().types.extend(types);

        Ok(hir::Use {
            keyword: self.keyword.clone(),
            path: self.path.clone(),
            imported_item,
        })
    }
}

impl ToHIR for ast::Module {
    type HIR = hir::ModuleData;
    type Error = ErrVec<Error>;

    /// Lower [`ast::Module`] to [`hir::Module`] within lowering context
    ///
    /// # Order
    ///
    /// 1. Use statements
    /// 2. Declare Types & Traits
    /// 3. Define Types
    /// 4. Declare Functions & Global variables
    /// 5. Define Traits
    /// 6. Define Functions & Global
    /// 7. Rest of statements
    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        use ast::Declaration as D;
        use ast::Statement as S;

        let mut errors = Vec::new();

        // Import things first
        self.statements
            .iter()
            .filter(|s| matches!(s, ast::Statement::Use(_)))
            .for_each(|stmt: &S| {
                let res = stmt.to_hir(context);
                match res {
                    Ok(mut stmt) => {
                        stmt.monomorphize(context);
                        context.module_mut().statements.push(stmt)
                    }
                    Err(err) => errors.push(err),
                }
            });

        let mut decls = HashMap::new();

        macro_rules! declare {
            () => {
                |stmt: &S| {
                    let decl: &D = match stmt {
                        S::Declaration(d) => d,
                        _ => return,
                    };

                    let res = decl.declare(context);
                    match res {
                        Ok(decl) => {
                            decls.insert(stmt.start(), decl);
                        }
                        Err(err) => {
                            errors.push(err);
                        }
                    }
                }
            };
        }

        macro_rules! define {
            () => {
                |stmt: &S| {
                    let decl: &D = match stmt {
                        S::Declaration(d) => d,
                        _ => return,
                    };

                    let res = decl.define(decls.remove(&decl.start()).unwrap(), context);
                    match res {
                        Ok(mut stmt) => {
                            stmt.monomorphize(context);
                            context.module_mut().statements.push(stmt.into())
                        }
                        Err(err) => errors.push(err),
                    }
                }
            };
        }

        // Declare Types & Traits
        self.statements
            .iter()
            .filter(|s| matches!(s, S::Declaration(D::Type(_) | D::Trait(_))))
            .for_each(declare!());

        // Define Types
        self.statements
            .iter()
            .filter(|s| matches!(s, S::Declaration(D::Type(_))))
            .for_each(define!());

        // Declare Functions & Global variables
        self.statements
            .iter()
            .filter(|s| matches!(s, S::Declaration(D::Function(_) | D::Variable(_))))
            .for_each(declare!());

        // Define Traits
        self.statements
            .iter()
            .filter(|s| matches!(s, S::Declaration(D::Trait(_))))
            .for_each(define!());

        // Define Functions & Global variables
        self.statements
            .iter()
            .filter(|s| matches!(s, S::Declaration(D::Function(_) | D::Variable(_))))
            .for_each(define!());

        // Add rest of statements
        self.statements
            .iter()
            .filter(|s| !matches!(s, S::Use(_) | S::Declaration(_)))
            .for_each(|stmt: &S| {
                let res = stmt.to_hir(context);
                match res {
                    Ok(mut stmt) => {
                        stmt.monomorphize(context);
                        context.module_mut().statements.push(stmt)
                    }
                    Err(err) => errors.push(err),
                }
            });

        if !errors.is_empty() {
            return Err(errors.into());
        }

        Ok(context.module().clone())
    }
}

/// Trait to replace [`TypeReference`] with type info
pub trait ReplaceWithTypeInfo {
    /// Replace [`TypeReference`] with type info
    fn replace_with_type_info(&self, context: &mut impl Context) -> hir::Expression;
}

impl ReplaceWithTypeInfo for TypeReference {
    fn replace_with_type_info(&self, context: &mut impl Context) -> hir::Expression {
        if self.is_generic() {
            return self.clone().into();
        }

        let name = self.type_for_type.name().to_string();
        let variable = if let Some(var) = context.module().find_variable(&name) {
            var
        } else {
            let var = Variable::new(VariableData {
                keyword: Keyword::<"let">::at(self.start()),
                mutability: Mutability::Immutable,
                name: Identifier::from(name).at(self.start()),
                ty: self.type_for_type.clone(),
                type_reference: None,
                initializer: Some(
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
                                    value: self.referenced_type.name().to_string(),
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
                    .into(),
                ),
            });
            context.module_mut().add_variable(var.clone());
            var.into()
        };

        hir::VariableReference {
            span: self.range(),
            variable,
        }
        .into()
    }
}

impl ToHIR for ast::GenericParameter {
    type HIR = Type;
    type Error = Error;

    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        Ok(GenericType {
            name: self.name.clone(),
            generated: false,
            constraint: self
                .constraint
                .as_ref()
                .map(|ty| ty.to_hir(context))
                .transpose()?,
        }
        .into())
    }
}

impl<T: ToHIR> ToHIR for Vec<T> {
    type HIR = Vec<T::HIR>;
    type Error = T::Error;

    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Self::Error> {
        self.iter().map(|t| t.to_hir(context)).try_collect()
    }
}
