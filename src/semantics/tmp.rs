use std::iter;

use derive_visitor::VisitorMut;

use crate::{
    hir::{
        Block, Declaration, Expression, ImplicitConversion, ImplicitConversionKind, Return,
        Statement, Typed, Variable, VariableData, VariableReference,
    },
    mutability::Mutable,
    syntax::{Identifier, Keyword, Ranged},
};

#[derive(VisitorMut)]
#[visitor(Statement(exit), Return(exit), ImplicitConversion(exit))]
pub struct TemporariesInserter {
    temporaries: Vec<Variable>,
}

impl<'ctx> TemporariesInserter {
    pub fn new() -> Self {
        Self {
            temporaries: Vec::new(),
        }
    }

    fn replace_with_tmp(&mut self, expr: &mut Expression) {
        let offset = expr.start();
        let tmp = Variable::new(VariableData {
            keyword: Keyword::<"let">::at(offset),
            mutability: expr.mutability(),
            name: Identifier::from(format!("$tmp@{offset}")).at(offset),
            type_reference: None,
            ty: expr.ty(),
            initializer: Some(expr.clone()),
        });
        *expr = VariableReference {
            span: expr.range(),
            variable: tmp.clone().into(),
        }
        .into();
        self.temporaries.push(tmp);
    }

    fn exit_implicit_conversion(&mut self, conv: &mut ImplicitConversion) {
        if conv.kind != ImplicitConversionKind::Reference
            || matches!(
                *conv.expression,
                Expression::VariableReference(_) | Expression::MemberReference(_)
            )
        {
            return;
        }

        self.replace_with_tmp(&mut conv.expression)
    }

    fn exit_return(&mut self, ret: &mut Return) {
        ret.value_mut().map(|expr| self.replace_with_tmp(expr));
    }

    fn exit_statement(&mut self, stmt: &mut Statement) {
        if self.temporaries.is_empty() {
            return;
        }

        *stmt = Block {
            statements: self
                .temporaries
                .drain(..)
                .map(Declaration::from)
                .map(Statement::from)
                .chain(iter::once(stmt.clone()))
                .collect(),
        }
        .into()
    }
}
