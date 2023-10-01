use std::sync::Arc;

use crate::{
    ast,
    hir::{self, FunctionDefinition, Typed},
    syntax::Ranged,
};

use super::{
    error::{CantDeduceReturnType, Error, ReturnTypeMismatch},
    ASTLoweringWithinContext, Context, FunctionContext, TraitContext,
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
    type Declaration = Arc<hir::FunctionDeclaration>;
    type Definition = hir::Function;

    fn declare(&self, context: &mut impl Context) -> Result<Self::Declaration, Error> {
        let mut name_parts: Vec<hir::FunctionNamePart> = Vec::new();
        for part in &self.name_parts {
            match part {
                ast::FunctionNamePart::Text(t) => name_parts.push(t.clone().into()),
                ast::FunctionNamePart::Parameter { parameter, .. } => {
                    name_parts.push(parameter.lower_to_hir_within_context(context)?.into())
                }
            }
        }

        let return_type = match &self.return_type {
            Some(ty) => ty.lower_to_hir_within_context(context)?.referenced_type,
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
            parent: context,
        };

        let mut body = Vec::new();
        for stmt in &self.body {
            body.push(stmt.lower_to_hir_within_context(&mut f_context)?);
        }

        if self.implicit_return {
            let return_type = f_context.function.return_type.clone();
            let expr: hir::Expression = body.pop().unwrap().try_into().unwrap();
            if self.return_type.is_none() {
                // One reference is held by module
                // Another reference is held by declaration itself
                // Last reference is inside context
                if Arc::strong_count(&declaration) > 3 {
                    return Err(CantDeduceReturnType {
                        at: self.name_parts.range().into(),
                    }
                    .into());
                } else {
                    unsafe {
                        (*Arc::as_ptr(&declaration).cast_mut()).return_type = expr.ty().clone();
                    }
                }
            } else {
                if expr.ty() != return_type {
                    return Err(ReturnTypeMismatch {
                        expected: return_type.clone(),
                        got: expr.ty(),
                        got_span: expr.range().into(),
                    }
                    .into());
                }
            }
            body = vec![hir::Return { value: Some(expr) }.into()];
        }

        let f = Arc::new(FunctionDefinition { declaration, body });

        context.add_function(f.clone().into());

        Ok(f.into())
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
                    functions: vec![],
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
                    functions: vec![],
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
        });

        if let Some(error) = error {
            return Err(error);
        }

        context.add_trait(tr.clone());

        Ok(tr)
    }
}