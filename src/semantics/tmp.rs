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
#[visitor(Expression, Statement)]
pub struct TemporariesInserter {
    temporaries: Vec<Variable>,
    depth: usize,
    is_in_assignment_or_var: bool,
}

impl TemporariesInserter {
    pub fn new() -> Self {
        Self {
            temporaries: Vec::new(),
            depth: 0,
            is_in_assignment_or_var: false,
        }
    }

    fn enter_expression(&mut self, _: &Expression) {
        self.depth += 1;
    }

    fn exit_expression(&mut self, expr: &mut Expression) {
        self.depth -= 1;

        if matches!(
            expr,
            Expression::VariableReference(_) | Expression::MemberReference(_)
        ) || self.is_in_assignment_or_var && self.depth == 0
        {
            return;
        }

        let index = self.temporaries.len();
        let offset = expr.start();
        let tmp = Variable::new(VariableData {
            keyword: Keyword::<"let">::at(offset),
            mutability: Mutability::Immutable,
            name: Identifier::from(format!("$tmp{index}@{offset}")).at(offset),
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

    fn enter_statement(&mut self, stmt: &Statement) {
        self.is_in_assignment_or_var = matches!(
            stmt,
            Statement::Assignment(_) | Statement::Declaration(Declaration::Variable(_))
        );
    }

    fn exit_statement(&mut self, stmt: &mut Statement) {
        self.is_in_assignment_or_var = false;

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
