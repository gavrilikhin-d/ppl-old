use std::sync::Arc;

use log::{debug, trace};

use crate::{
    hir::{
        Assignment, Call, ClassDeclaration, Constructor, Declaration, ElseIf, Expression, Function,
        FunctionData, FunctionNamePart, Generic, If, ImplicitConversion, ImplicitConversionKind,
        Initializer, Loop, Member, MemberReference, Module, Parameter, ParameterOrVariable, Return,
        Statement, Type, TypeReference, Typed, Variable, VariableData, VariableReference, While,
    },
    semantics::{ConvertibleTo, GenericContext, Implicit},
};

use super::{Context, ReplaceWithTypeInfo};

/// Trait to get monomorphized version of statements
pub trait Monomorphized {
    /// Get monomorphized version of statement
    fn monomorphized(self, context: &mut impl Context) -> Self;
}

impl<T: Monomorphized> Monomorphized for Vec<T> {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        self.into_iter()
            .map(|val| val.monomorphized(context))
            .collect()
    }
}

impl Monomorphized for Statement {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            Statement::Expression(expr) => expr.monomorphized(context).into(),
            Statement::Assignment(a) => a.monomorphized(context).into(),
            Statement::If(stmt) => stmt.monomorphized(context).into(),
            Statement::Loop(l) => l.monomorphized(context).into(),
            Statement::While(l) => l.monomorphized(context).into(),
            Statement::Return(ret) => ret.monomorphized(context).into(),
            Statement::Declaration(d) => d.monomorphized(context).into(),
            Statement::Use(_) => self,
        }
    }
}

impl Monomorphized for Declaration {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            Declaration::Variable(v) => v.monomorphized(context).into(),
            _ => self,
        }
    }
}

impl Monomorphized for Variable {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic()
            && !self
                .read()
                .unwrap()
                .initializer
                .as_ref()
                .map(Generic::is_generic)
                .unwrap_or(false)
        {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        let res = Variable::new(VariableData {
            ty: self.ty().clone().monomorphized(context),
            initializer: self
                .read()
                .unwrap()
                .initializer
                .clone()
                .map(|i| i.monomorphized(context)),
            ..self.read().unwrap().clone()
        });

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for Assignment {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        Assignment {
            target: self.target.monomorphized(context),
            value: self.value.monomorphized(context),
        }
    }
}

impl Monomorphized for If {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        If {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
            else_ifs: self.else_ifs.monomorphized(context),
            else_block: self.else_block.monomorphized(context),
        }
    }
}

impl Monomorphized for ElseIf {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        ElseIf {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for Return {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        Return {
            value: self.value.clone().map(|value| value.monomorphized(context)),
        }
    }
}

impl Monomorphized for Loop {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        Loop {
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for While {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        While {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for ImplicitConversion {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        let expression = Box::new(self.expression.monomorphized(context));
        use ImplicitConversionKind::*;
        match self.kind {
            Reference => expression.reference(context).try_into().unwrap(),
            Dereference => expression.dereference().try_into().unwrap(),
        }
    }
}

impl Monomorphized for Expression {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            Expression::Call(c) => c.monomorphized(context).into(),
            Expression::VariableReference(var) => var.monomorphized(context).into(),
            Expression::TypeReference(ty) => ty
                .monomorphized(context)
                .replace_with_type_info(context)
                .into(),
            Expression::Literal(_) => self,
            Expression::MemberReference(m) => m.monomorphized(context).into(),
            Expression::Constructor(c) => c.monomorphized(context).into(),
            Expression::ImplicitConversion(c) => c.monomorphized(context).into(),
        }
    }
}

impl Monomorphized for Constructor {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        let initializers = self.initializers.monomorphized(context);
        let res = Constructor {
            ty: self.ty.monomorphized(context),
            initializers,
            rbrace: self.rbrace,
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for Initializer {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        Initializer {
            member: self.member.monomorphized(context),
            value: self.value.monomorphized(context),
            ..self
        }
    }
}

impl Monomorphized for TypeReference {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        let referenced_type = self.referenced_type.monomorphized(context);
        let res = TypeReference {
            referenced_type: referenced_type.clone(),
            type_for_type: context.builtin().types().type_of(referenced_type),
            ..self
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for Type {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            Type::Class(c) => c.monomorphized(context).into(),
            Type::Function(_) => todo!(),
            Type::Generic(_) | Type::SelfType(_) | Type::Trait(_) => {
                context.get_specialized(self.clone()).unwrap_or(self)
            }
            Type::Unknown => unreachable!("Trying to monomorphize not-inferred type"),
        }
    }
}

impl Monomorphized for Arc<ClassDeclaration> {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "\n{self:#}");
            return self;
        }

        let from = format!("{:#}", self);
        trace!(target: "monomorphizing", "\n{from}");

        let res = Arc::new(ClassDeclaration {
            generic_parameters: self.generic_parameters.clone().monomorphized(context),
            members: self.members.clone().monomorphized(context),
            ..self.as_ref().clone()
        });

        debug!(target: "monomorphized-from", "\n{from}");
        debug!(target: "monomorphized-to", "\n{res:#}");

        res
    }
}

impl Monomorphized for Arc<Member> {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self;
        }

        Arc::new(Member {
            ty: self.ty.clone().monomorphized(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphized for Call {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");
        let args = self.args.monomorphized(context);

        let mut context = GenericContext::for_fn(self.function.clone(), context);

        args.iter()
            .map(|arg| arg.ty())
            .zip(self.function.read().unwrap().parameters().map(|p| p.ty()))
            .for_each(|(arg, p)| {
                arg.convertible_to(p).within(&mut context).unwrap();
            });

        let res = Call {
            function: self.function.monomorphized(&mut context),
            args,
            ..self
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for VariableReference {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");
        let res = VariableReference {
            span: self.span,
            variable: self.variable.monomorphized(context),
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for ParameterOrVariable {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            ParameterOrVariable::Variable(v) => v.monomorphized(context).into(),
            ParameterOrVariable::Parameter(p) => p.monomorphized(context).into(),
        }
    }
}

impl Monomorphized for Function {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        let this = self.read().unwrap();

        if !this.is_generic() {
            trace!(target: "monomorphizing-skipped", "{this}");
            drop(this);
            return self;
        }

        let from = this.to_string();
        trace!(target: "monomorphizing", "{from}");

        let generic_types = this.generic_types.clone().monomorphized(context);
        let name_parts = this.name_parts.clone().monomorphized(context);
        let name = FunctionData::build_name(&name_parts);
        let return_type = this.return_type.clone().monomorphized(context);
        let mut f = FunctionData::build()
            .with_generic_types(generic_types)
            .with_mangled_name(
                context
                    .function_with_name(&name)
                    .map(|f| f.read().unwrap().mangled_name.clone())
                    .flatten()
                    .or_else(|| this.mangled_name.clone()),
            )
            .with_name(name_parts)
            .with_return_type(return_type);

        f.body = this.body.clone().monomorphized(context);

        let ty_for_self =
            this.name_parts()
                .iter()
                .zip(f.name_parts())
                .find_map(|parts| match parts {
                    (
                        FunctionNamePart::Parameter(original),
                        FunctionNamePart::Parameter(mapped),
                    ) => match original.ty() {
                        Type::SelfType(_) if !mapped.ty().is_generic() => Some(mapped.ty()),
                        _ => None,
                    },
                    _ => None,
                });

        // Find real implementation of trait function after monomorphization
        if !f.is_definition()
            && let Some(ty_for_self) = ty_for_self
            && let Some(real_impl) = context.find_implementation(&this, &ty_for_self)
        {
            // FIXME: Need to do this shit, because generic types in `real_impl` aren't the same as in `f`
            let mut context = GenericContext::for_fn(real_impl.clone(), context);
            f.parameters()
                .map(|arg| arg.ty())
                .zip(real_impl.read().unwrap().parameters().map(|p| p.ty()))
                .for_each(|(arg, p)| {
                    arg.convertible_to(p).within(&mut context).unwrap();
                });

            f = real_impl
                .monomorphized(&mut context)
                .read()
                .unwrap()
                .clone();
        }

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{f}");

        Function::new(f)
    }
}

impl Monomorphized for FunctionNamePart {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        match self {
            FunctionNamePart::Text(_) => self,
            FunctionNamePart::Parameter(p) => p.monomorphized(context).into(),
        }
    }
}

impl Monomorphized for Arc<Parameter> {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self;
        }

        Arc::new(Parameter {
            ty: self.ty.clone().monomorphized(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphized for MemberReference {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        let base = Box::new(self.base.monomorphized(context));
        let member = base.ty().members()[self.index].clone();

        let res = MemberReference {
            base,
            member,
            ..self
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphized for Module {
    fn monomorphized(self, context: &mut impl Context) -> Self {
        trace!(target: "monomorphizing", "{}", context.module().source_file().name());

        let statements = self.statements.monomorphized(context);
        let mut variables = self.variables;
        variables.values_mut().for_each(|v| {
            *v = v.clone().monomorphized(context);
        });
        Module {
            variables,
            statements,
            ..self
        }
    }
}
