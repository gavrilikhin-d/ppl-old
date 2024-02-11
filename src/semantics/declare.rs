use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;

use crate::{
    ast,
    hir::{self, Function, Type, Typed},
    syntax::Ranged,
};

use super::{
    error::{CantDeduceReturnType, Error, ReturnTypeMismatch},
    Context, ConvertibleTo, FunctionContext, GenericContext, Monomorphize, ToHIR, TraitContext,
};

/// Trait to pre-declare something
pub trait Declare {
    type Declaration;
    type Definition;

    /// Declare entity in context
    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error>;

    /// Define entity in context
    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error>;
}

impl Declare for ast::FunctionDeclaration {
    type Declaration = Function;
    type Definition = Function;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        // TODO: check for collision
        let generic_parameters: Vec<Type> = self.generic_parameters.to_hir(context)?;

        let mut generic_context = GenericContext {
            parent: context,
            generic_parameters: generic_parameters.clone(),
            generics_mapping: HashMap::new(),
        };

        let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
        for part in &self.name_parts {
            match part {
                ast::FunctionNamePart::Text(t) => name_parts.push(t.clone().into()),
                ast::FunctionNamePart::Parameter(p) => {
                    name_parts.push(p.to_hir(&mut generic_context)?.into())
                }
            }
        }

        let return_type = match &self.return_type {
            Some(ty) => ty.to_hir(&mut generic_context)?.referenced_type,
            None if self.implicit_return => Type::Unknown,
            None => generic_context.builtin().types().none(),
        };

        // Copy generic parameters from generic context, as we may have added new parameters
        let generic_parameters = generic_context.generic_parameters.clone();

        drop(generic_context);

        // TODO: error if invalid annotation
        let annotations = self
            .annotations
            .iter()
            .map(|a| a.to_hir(context))
            .collect::<Result<Vec<_>, _>>()?;
        let mangled_name = annotations.iter().find_map(|a| match a {
            hir::Annotation::MangleAs(name) => Some(name.clone()),
            _ => None,
        });

        let f = Function::new(
            hir::FunctionData::build(self.keyword)
                .with_generic_types(generic_parameters)
                .with_name(name_parts)
                .with_mangled_name(mangled_name)
                .with_return_type(return_type),
        );

        context.add_function(f.clone());

        Ok(f)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        if self.body.is_empty() {
            return Ok(declaration.into());
        }

        let mut f_context = FunctionContext {
            function: declaration.clone(),
            variables: vec![],
            parent: context,
        };

        let mut body = Vec::new();
        for stmt in &self.body {
            body.push(stmt.to_hir(&mut f_context)?);
        }

        if self.implicit_return {
            let return_type = f_context.function.read().unwrap().return_type.clone();
            let expr: hir::Expression = body.pop().unwrap().try_into().unwrap();
            if self.return_type.is_none() {
                if expr.ty() == Type::Unknown {
                    return Err(CantDeduceReturnType {
                        at: self.name_parts.range().into(),
                    }
                    .into());
                }

                declaration.write().unwrap().return_type = expr.ty().clone();
            } else if !expr
                .ty()
                .convertible_to(return_type.clone())
                .within(context)
                .is_ok_and(|convertible| convertible)
            {
                return Err(ReturnTypeMismatch {
                    expected: return_type.clone(),
                    got: expr.ty(),
                    got_span: expr.range().into(),
                }
                .into());
            }
            body = vec![hir::Return::Implicit { value: expr }.into()];
        }

        declaration.write().unwrap().body = body;

        Ok(declaration)
    }
}

impl Declare for ast::TraitDeclaration {
    type Declaration = Arc<hir::TraitDeclaration>;
    type Definition = Arc<hir::TraitDeclaration>;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let mut error = None;
        let tr = Arc::new_cyclic(|trait_weak| {
            let mut context = TraitContext {
                tr: hir::TraitDeclaration {
                    name: self.name.clone(),
                    functions: IndexMap::new(),
                },
                trait_weak: trait_weak.clone(),
                parent: context,
            };

            for f in &self.functions {
                error = f.declare(&mut context).err();
                if error.is_some() {
                    break;
                }
            }

            context.tr
        });

        if let Some(error) = error {
            return Err(error);
        }

        context.add_trait(tr.clone());

        Ok(tr)
    }

    fn define(
        &self,
        _declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        let mut error = None;
        let tr = Arc::new_cyclic(|trait_weak| {
            let mut context = TraitContext {
                tr: hir::TraitDeclaration {
                    name: self.name.clone(),
                    functions: IndexMap::new(),
                },
                trait_weak: trait_weak.clone(),
                parent: context,
            };

            for f in &self.functions {
                error = f.to_hir(&mut context).err();
                if error.is_some() {
                    break;
                }
            }

            context.tr
        });

        if let Some(error) = error {
            return Err(error);
        }

        context.add_trait(tr.clone());

        Ok(tr)
    }
}

impl Declare for ast::TypeDeclaration {
    type Declaration = Arc<hir::ClassDeclaration>;
    type Definition = Arc<hir::ClassDeclaration>;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let annotations = self
            .annotations
            .iter()
            .map(|a| a.to_hir(context))
            .collect::<Result<Vec<_>, _>>()?;
        let is_builtin = annotations
            .iter()
            .any(|a| matches!(a, hir::Annotation::Builtin));
        // TODO: error for incorrect builtin type name
        let builtin = if is_builtin {
            Some(self.name.parse().unwrap())
        } else {
            None
        };

        // TODO: check for collisions, etc
        let generic_parameters: Vec<Type> = self.generic_parameters.to_hir(context)?;

        // TODO: recursive types
        let ty = Arc::new(hir::ClassDeclaration {
            basename: self.name.clone(),
            specialization_of: None,
            generic_parameters,
            builtin,
            members: vec![],
        });

        context.add_type(ty.clone());

        Ok(ty)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        let mut generic_context = GenericContext {
            parent: context,
            generic_parameters: declaration.generic_parameters.clone(),
            generics_mapping: HashMap::new(),
        };

        // TODO: recursive types
        let ty = Arc::new(hir::ClassDeclaration {
            members: self
                .members
                .iter()
                .map(|m| m.to_hir(&mut generic_context))
                .try_collect()?,
            ..(*declaration).clone()
        });

        context.add_type(ty.clone());

        Ok(ty)
    }
}

impl Declare for ast::VariableDeclaration {
    type Declaration = hir::Variable;
    type Definition = hir::Variable;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let var = hir::Variable::new(hir::VariableData {
            name: self.name.clone(),
            ty: Type::Unknown,
            initializer: None,
            mutability: self.mutability.clone(),
        });

        context.add_variable(var.clone());

        Ok(var)
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        let mut initializer = self.initializer.to_hir(context)?;
        initializer.monomorphize(context);

        declaration.write().unwrap().ty = initializer.ty();
        declaration.write().unwrap().initializer = Some(initializer);

        Ok(declaration)
    }
}

impl Declare for ast::Declaration {
    type Declaration = hir::Declaration;
    type Definition = hir::Declaration;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        match self {
            ast::Declaration::Function(f) => f.declare(context).map(Into::into),
            ast::Declaration::Trait(t) => t.declare(context).map(Into::into),
            ast::Declaration::Type(t) => t.declare(context).map(Into::into),
            ast::Declaration::Variable(v) => v.declare(context).map(Into::into),
        }
    }

    fn define(
        &self,
        declaration: Self::Declaration,
        context: &mut impl Context,
    ) -> Result<Self::Definition, Error> {
        match self {
            ast::Declaration::Function(f) => f
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Trait(t) => t
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Type(t) => t
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
            ast::Declaration::Variable(v) => v
                .define(declaration.try_into().unwrap(), context)
                .map(Into::into),
        }
    }
}

impl<D: Declare> ToHIR for D {
    type HIR = D::Definition;

    fn to_hir(&self, context: &mut impl Context) -> Result<Self::HIR, Error> {
        self.define(self.declare(context)?, context)
    }
}
