use std::{collections::HashMap, sync::Arc};

use crate::{
    hir::{
        Assignment, Call, Constructor, ElseIf, Expression, Function, FunctionDeclaration,
        FunctionDefinition, FunctionNamePart, Generic, If, ImplicitConversion, Initializer, Loop,
        Member, MemberReference, Parameter, Return, Specialize, Statement, Type, TypeDeclaration,
        TypeReference, Typed, VariableReference, While,
    },
    named::Named,
    semantics::FunctionContext,
};

use super::{Context, ReplaceWithTypeInfo};

/// Trait to get monomorphized version of statements
pub trait Monomorphized {
    /// Get monomorphized version of statement
    fn monomorphized(&self, context: &mut impl Context) -> Self;
}

impl<T: Monomorphized> Monomorphized for Vec<T> {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        self.iter().map(|val| val.monomorphized(context)).collect()
    }
}

impl Monomorphized for Statement {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        match self {
            Statement::Expression(expr) => expr.monomorphized(context).into(),
            Statement::Assignment(a) => a.monomorphized(context).into(),
            Statement::If(stmt) => stmt.monomorphized(context).into(),
            Statement::Loop(l) => l.monomorphized(context).into(),
            Statement::While(l) => l.monomorphized(context).into(),
            Statement::Return(ret) => ret.monomorphized(context).into(),

            // Declarations only monomorphized when referenced
            Statement::Declaration(_) | Statement::Use(_) => self.clone(),
        }
    }
}

impl Monomorphized for Assignment {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Assignment {
            target: self.target.monomorphized(context),
            value: self.value.monomorphized(context),
        }
    }
}

impl Monomorphized for If {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        If {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
            else_ifs: self.else_ifs.monomorphized(context),
            else_block: self.else_block.monomorphized(context),
        }
    }
}

impl Monomorphized for ElseIf {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        ElseIf {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for Return {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Return {
            value: self.value.clone().map(|value| value.monomorphized(context)),
        }
    }
}

impl Monomorphized for Loop {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Loop {
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for While {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        While {
            condition: self.condition.monomorphized(context),
            body: self.body.monomorphized(context),
        }
    }
}

impl Monomorphized for ImplicitConversion {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        ImplicitConversion {
            kind: self.kind.clone(),
            ty: self.ty.clone(),
            expression: Box::new(self.expression.monomorphized(context)),
        }
    }
}

impl Monomorphized for Expression {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        match self {
            Expression::Call(c) => c.monomorphized(context).into(),
            Expression::VariableReference(var) => var.monomorphized(context).into(),
            Expression::TypeReference(ty) => ty
                .monomorphized(context)
                .replace_with_type_info(context)
                .into(),
            Expression::Literal(_) => self.clone(),
            Expression::MemberReference(m) => m.monomorphized(context).into(),
            Expression::Constructor(c) => c.monomorphized(context).into(),
            Expression::ImplicitConversion(c) => c.monomorphized(context).into(),
        }
    }
}

impl Monomorphized for Constructor {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Constructor {
            ty: self.ty.monomorphized(context),
            initializers: self.initializers.monomorphized(context),
            rbrace: self.rbrace,
        }
    }
}

impl Monomorphized for Initializer {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        Initializer {
            member: self.member.monomorphized(context),
            value: self.value.monomorphized(context),
            ..self.clone()
        }
    }
}

impl Monomorphized for TypeReference {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        let referenced_type = self.referenced_type.monomorphized(context);
        TypeReference {
            referenced_type: referenced_type.clone(),
            type_for_type: context.builtin().types().type_of(referenced_type),
            ..self.clone()
        }
    }
}

impl Monomorphized for Type {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        match self {
            Type::Class(c) => c.monomorphized(context).into(),
            Type::Function(_) => todo!(),
            Type::Generic(_) | Type::SelfType(_) | Type::Trait(_) => {
                context.get_specialized(self.clone()).unwrap()
            }
        }
    }
}

impl Monomorphized for Arc<TypeDeclaration> {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        Arc::new(TypeDeclaration {
            generic_parameters: self.generic_parameters.monomorphized(context),
            members: self.members.monomorphized(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphized for Arc<Member> {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        Arc::new(Member {
            ty: self.ty.monomorphized(context),
            ..self.as_ref().clone()
        })
    }
}

impl Monomorphized for Call {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        let args = self.args.monomorphized(context);
        Call {
            function: self
                .function
                .monomorphized(context, args.iter().map(|arg| arg.ty())),
            args,
            ..self.clone()
        }
    }
}

impl Monomorphized for VariableReference {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        VariableReference {
            span: self.span.clone(),
            variable: context.find_variable(&self.variable.name()).unwrap(),
        }
    }
}

/// Trait to get monomorphized version of function
pub trait MonomorphizedWithArgs {
    /// Get monomorphized version of function
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self;
}

impl MonomorphizedWithArgs for Arc<FunctionDeclaration> {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        // TODO: use context mapping for it
        let mut generics_map: HashMap<Type, Type> = HashMap::new();

        let mut arg = args.into_iter();
        let name_parts = self
            .name_parts()
            .iter()
            .map(|part| match part {
                FunctionNamePart::Text(text) => text.clone().into(),
                FunctionNamePart::Parameter(param) => {
                    let mut arg_ty = arg.next().unwrap().clone();
                    if !param.is_generic() {
                        return param.clone().into();
                    }

                    let param_ty = param.ty();
                    if param_ty.is_any_reference() && !arg_ty.is_any_reference() {
                        arg_ty = context.builtin().types().reference_to(arg_ty);
                    }

                    let diff = param_ty.diff(arg_ty.clone());
                    generics_map.extend(diff);

                    Arc::new(Parameter {
                        ty: TypeReference {
                            referenced_type: arg_ty,
                            ..param.ty.clone()
                        },
                        ..param.as_ref().clone()
                    })
                    .into()
                }
            })
            .collect::<Vec<_>>();

        let name = Function::build_name(&name_parts);

        let return_type = self.return_type.clone().specialize_with(&generics_map);

        let generic_types: Vec<Type> = self
            .generic_types
            .iter()
            .cloned()
            .map(|g| g.specialize_with(&generics_map))
            .collect();
        Arc::new(
            FunctionDeclaration::build()
                .with_generic_types(generic_types)
                .with_name(name_parts)
                .with_mangled_name(
                    context
                        .function_with_name(&name)
                        .map(|f| f.declaration().mangled_name.clone())
                        .flatten()
                        .or_else(|| self.mangled_name.clone()),
                )
                .with_return_type(return_type),
        )
    }
}

impl MonomorphizedWithArgs for Arc<FunctionDefinition> {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        let declaration = self.declaration.monomorphized(context, args);

        let mut context = FunctionContext {
            function: declaration.clone(),
            variables: vec![],
            parent: context,
        };

        let body = self
            .body
            .iter()
            .map(|stmt| stmt.monomorphized(&mut context))
            .collect();

        let f = Arc::new(FunctionDefinition { declaration, body });

        f
    }
}

impl MonomorphizedWithArgs for Function {
    fn monomorphized(
        &self,
        context: &mut impl Context,
        args: impl IntoIterator<Item = Type>,
    ) -> Self {
        match self {
            Function::Declaration(d) => d.monomorphized(context, args).into(),
            Function::Definition(d) => d.monomorphized(context, args).into(),
        }
    }
}

impl Monomorphized for MemberReference {
    fn monomorphized(&self, context: &mut impl Context) -> Self {
        if !self.is_generic() {
            return self.clone();
        }

        let base = Box::new(self.base.monomorphized(context));
        let member = base.ty().members()[self.index].clone();

        MemberReference {
            base,
            member,
            ..(self.clone())
        }
    }
}
