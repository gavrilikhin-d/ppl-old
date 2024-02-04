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
pub trait Monomorphize {
    /// Get monomorphized version of statement
    fn monomorphize(self, context: &mut impl Context) -> Self;
}

impl<T: Monomorphize> Monomorphize for Vec<T> {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        self.into_iter()
            .map(|val| val.monomorphize(context))
            .collect()
    }
}

impl Monomorphize for Statement {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            Statement::Expression(expr) => expr.monomorphize(context).into(),
            Statement::Assignment(a) => a.monomorphize(context).into(),
            Statement::If(stmt) => stmt.monomorphize(context).into(),
            Statement::Loop(l) => l.monomorphize(context).into(),
            Statement::While(l) => l.monomorphize(context).into(),
            Statement::Return(ret) => ret.monomorphize(context).into(),
            Statement::Declaration(d) => d.monomorphize(context).into(),
            Statement::Use(_) => self,
        }
    }
}

impl Monomorphize for Declaration {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            Declaration::Variable(v) => v.monomorphize(context).into(),
            _ => self,
        }
    }
}

impl Monomorphize for Variable {
    fn monomorphize(self, context: &mut impl Context) -> Self {
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
            ty: self.ty().clone().monomorphize(context),
            initializer: self
                .read()
                .unwrap()
                .initializer
                .clone()
                .map(|i| i.monomorphize(context)),
            ..self.read().unwrap().clone()
        });

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphize for Assignment {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        Assignment {
            target: self.target.monomorphize(context),
            value: self.value.monomorphize(context),
        }
    }
}

impl Monomorphize for If {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        If {
            condition: self.condition.monomorphize(context),
            body: self.body.monomorphize(context),
            else_ifs: self.else_ifs.monomorphize(context),
            else_block: self.else_block.monomorphize(context),
        }
    }
}

impl Monomorphize for ElseIf {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        ElseIf {
            condition: self.condition.monomorphize(context),
            body: self.body.monomorphize(context),
        }
    }
}

impl Monomorphize for Return {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        Return {
            value: self.value.clone().map(|value| value.monomorphize(context)),
        }
    }
}

impl Monomorphize for Loop {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        Loop {
            body: self.body.monomorphize(context),
        }
    }
}

impl Monomorphize for While {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        While {
            condition: self.condition.monomorphize(context),
            body: self.body.monomorphize(context),
        }
    }
}

impl Monomorphize for ImplicitConversion {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        let expression = Box::new(self.expression.monomorphize(context));
        use ImplicitConversionKind::*;
        match self.kind {
            Reference => expression.reference(context).try_into().unwrap(),
            Dereference => expression.dereference().try_into().unwrap(),
        }
    }
}

impl Monomorphize for Expression {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            Expression::Call(c) => c.monomorphize(context).into(),
            Expression::VariableReference(var) => var.monomorphize(context).into(),
            Expression::TypeReference(ty) => ty
                .monomorphize(context)
                .replace_with_type_info(context)
                .into(),
            Expression::Literal(_) => self,
            Expression::MemberReference(m) => m.monomorphize(context).into(),
            Expression::Constructor(c) => c.monomorphize(context).into(),
            Expression::ImplicitConversion(c) => c.monomorphize(context).into(),
        }
    }
}

impl Monomorphize for Constructor {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");

        let initializers = self.initializers.monomorphize(context);
        let res = Constructor {
            ty: self.ty.monomorphize(context),
            initializers,
            rbrace: self.rbrace,
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphize for Initializer {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        Initializer {
            member: self.member.monomorphize(context),
            value: self.value.monomorphize(context),
            ..self
        }
    }
}

impl Monomorphize for TypeReference {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        let referenced_type = self.referenced_type.monomorphize(context);
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

impl Monomorphize for Type {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            Type::Class(c) => c.monomorphize(context).into(),
            Type::Function(_) => todo!(),
            Type::Generic(_) | Type::SelfType(_) | Type::Trait(_) => {
                context.get_specialized(self.clone()).unwrap_or(self)
            }
            Type::Unknown => unreachable!("Trying to monomorphize not-inferred type"),
        }
    }
}

impl Monomorphize for Arc<ClassDeclaration> {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "\n{self:#}");
            return self;
        }

        let from = format!("{:#}", self);
        trace!(target: "monomorphizing", "\n{from}");

        let res = Arc::new(ClassDeclaration {
            generic_parameters: self.generic_parameters.clone().monomorphize(context),
            members: self.members.clone().monomorphize(context),
            ..self.as_ref().clone()
        });

        debug!(target: "monomorphized-from", "\n{from}");
        debug!(target: "monomorphized-to", "\n{res:#}");

        res
    }
}

impl Monomorphize for Arc<Member> {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self;
        }

        Arc::new(Member {
            ty: self.ty.clone().monomorphize(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphize for Call {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{from}");
        let args = self.args.monomorphize(context);

        let mut context = GenericContext::for_fn(self.function.clone(), context);

        args.iter()
            .map(|arg| arg.ty())
            .zip(self.function.read().unwrap().parameters().map(|p| p.ty()))
            .for_each(|(arg, p)| {
                arg.convertible_to(p).within(&mut context).unwrap();
            });

        let res = Call {
            function: self.function.monomorphize(&mut context),
            args,
            ..self
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphize for VariableReference {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");
        let res = VariableReference {
            span: self.span,
            variable: self.variable.monomorphize(context),
        };

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{res}");

        res
    }
}

impl Monomorphize for ParameterOrVariable {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            ParameterOrVariable::Variable(v) => v.monomorphize(context).into(),
            ParameterOrVariable::Parameter(p) => p.monomorphize(context).into(),
        }
    }
}

impl Monomorphize for Function {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        let this = self.read().unwrap();

        if !this.is_generic() {
            trace!(target: "monomorphizing-skipped", "{this}");
            drop(this);
            return self;
        }

        let from = this.to_string();
        trace!(target: "monomorphizing", "{from}");

        let generic_types = this.generic_types.clone().monomorphize(context);
        let name_parts = this.name_parts.clone().monomorphize(context);
        let name = FunctionData::build_name(&name_parts);
        let return_type = this.return_type.clone().monomorphize(context);
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

        f.body = this.body.clone().monomorphize(context);

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

            f = real_impl.monomorphize(&mut context).read().unwrap().clone();
        }

        debug!(target: "monomorphized-from", "{from}");
        debug!(target: "monomorphized-to", "{f}");

        Function::new(f)
    }
}

impl Monomorphize for FunctionNamePart {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        match self {
            FunctionNamePart::Text(_) => self,
            FunctionNamePart::Parameter(p) => p.monomorphize(context).into(),
        }
    }
}

impl Monomorphize for Arc<Parameter> {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self;
        }

        Arc::new(Parameter {
            ty: self.ty.clone().monomorphize(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphize for MemberReference {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            trace!(target: "monomorphizing-skipped", "{self}");
            return self;
        }

        let from = self.to_string();
        trace!(target: "monomorphizing", "{self}");

        let base = Box::new(self.base.monomorphize(context));
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

impl Monomorphize for Module {
    fn monomorphize(self, context: &mut impl Context) -> Self {
        trace!(target: "monomorphizing", "{}", context.module().source_file().name());

        let statements = self.statements.monomorphize(context);
        let mut variables = self.variables;
        variables.values_mut().for_each(|v| {
            *v = v.clone().monomorphize(context);
        });
        Module {
            variables,
            statements,
            ..self
        }
    }
}
