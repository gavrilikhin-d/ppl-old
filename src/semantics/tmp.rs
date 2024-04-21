use std::iter;

use derive_visitor::VisitorMut;

use crate::{
    hir::{
        self, Block, Declaration, Expression, Statement, Typed, Variable, VariableData,
        VariableReference,
    },
    mutability::Mutability,
    syntax::{Identifier, Keyword, Ranged},
};

enum InsideOf {
    Assignment,
    VariableDeclaration,
    Return,
}

#[derive(VisitorMut)]
#[visitor(Expression, Statement)]
pub struct TemporariesInserter {
    temporaries: Vec<Variable>,
    depth: usize,
    inside_of: Option<InsideOf>,
}

impl TemporariesInserter {
    pub fn new() -> Self {
        Self {
            temporaries: Vec::new(),
            depth: 0,
            inside_of: None,
        }
    }

    fn enter_expression(&mut self, _: &Expression) {
        self.depth += 1;
    }

    fn exit_expression(&mut self, expr: &mut Expression) {
        self.depth -= 1;

        if matches!(
            expr,
            Expression::VariableReference(_)
                | Expression::MemberReference(_)
                | Expression::ImplicitConversion(_)
        ) {
            return;
        }

        if self.depth == 0
            && matches!(self.inside_of, Some(InsideOf::Return))
            && matches!(expr, Expression::Literal(_))
        {
            return;
        }

        if self.depth == 0
            && matches!(
                self.inside_of,
                Some(InsideOf::Assignment | InsideOf::VariableDeclaration)
            )
        {
            return;
        }

        if expr.ty().is_none() && !matches!(self.inside_of, Some(InsideOf::Return)) {
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
        use Statement::*;
        self.inside_of = match stmt {
            Return(_) => Some(InsideOf::Return),
            Assignment(_) => Some(InsideOf::Assignment),
            Declaration(hir::Declaration::Variable(_)) => Some(InsideOf::VariableDeclaration),
            _ => None,
        }
    }

    fn exit_statement(&mut self, stmt: &mut Statement) {
        self.inside_of = None;

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
