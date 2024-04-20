use std::iter;

use derive_visitor::VisitorMut;

use crate::{
    hir::{
        Block, Declaration, Expression, Statement, Typed, Variable, VariableData, VariableReference,
    },
    mutability::Mutability,
    syntax::{Identifier, Keyword, Ranged},
};

#[derive(VisitorMut)]
#[visitor(Expression(exit), Statement(exit))]
pub struct TemporariesInserter {
    temporaries: Vec<Variable>,
}

impl TemporariesInserter {
    pub fn new() -> Self {
        Self {
            temporaries: Vec::new(),
        }
    }

    fn exit_expression(&mut self, expr: &mut Expression) {
        if matches!(
            expr,
            Expression::VariableReference(_) | Expression::MemberReference(_)
        ) {
            return;
        }

        let tmp = Variable::new(VariableData {
            keyword: Keyword::<"let">::at(expr.start()),
            mutability: Mutability::Immutable,
            name: Identifier::from(format!("$tmp{}", self.temporaries.len())).at(expr.start()),
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
