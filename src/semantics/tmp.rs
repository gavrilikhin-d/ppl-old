use std::iter;

use derive_visitor::VisitorMut;

use crate::{
    hir::{
        Block, Declaration, Return, Statement, Typed, Variable, VariableData, VariableReference,
    },
    mutability::Mutability,
    syntax::{Identifier, Keyword, Ranged},
};

#[derive(VisitorMut)]
#[visitor(Statement(exit), Return(enter))]
pub struct TemporariesInserter {
    temporaries: Vec<Variable>,
}

impl<'ctx> TemporariesInserter {
    pub fn new() -> Self {
        Self {
            temporaries: Vec::new(),
        }
    }

    fn enter_return(&mut self, ret: &mut Return) {
        if ret.value().is_none() {
            return;
        }

        let expr = ret.value_mut().unwrap();
        let offset = expr.start();
        let tmp = Variable::new(VariableData {
            keyword: Keyword::<"let">::at(offset),
            mutability: Mutability::Immutable,
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
