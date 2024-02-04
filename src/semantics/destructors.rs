use crate::{
    hir::{self, Call, Function, Statement, Typed},
    syntax::Ranged,
};

use super::Context;

/// Insert destructors calls to HIR
fn with_destructors(statements: &[Statement], context: &mut impl Context) -> Vec<Statement> {
    let mut new_statements = vec![];
    for stmt in statements {
        use Statement::*;
        match stmt {
            Assignment(a) => {
                if let Some(destructor) = context.destructor_for(a.target.ty()) {
                    new_statements.push(
                        hir::Expression::from(Call {
                            range: a.target.range(),
                            function: destructor,
                            generic: None,
                            // FIXME: create temporary variable,
                            // if it's complex expr
                            args: vec![a.target.clone()],
                        })
                        .into(),
                    )
                }
            }
            If(if_stmt) => {
                new_statements.push(
                    hir::If {
                        condition: if_stmt.condition.clone(),
                        body: with_destructors(&if_stmt.body, context),
                        else_block: with_destructors(&if_stmt.else_block, context),
                        else_ifs: if_stmt
                            .else_ifs
                            .iter()
                            .map(|else_if| hir::ElseIf {
                                condition: else_if.condition.clone(),
                                body: with_destructors(&else_if.body, context),
                            })
                            .collect(),
                    }
                    .into(),
                );
                continue;
            }
            Loop(l) => {
                new_statements.push(
                    hir::Loop {
                        body: with_destructors(&l.body, context),
                    }
                    .into(),
                );
                continue;
            }
            While(w) => {
                new_statements.push(
                    hir::While {
                        condition: w.condition.clone(),
                        body: with_destructors(&w.body, context),
                    }
                    .into(),
                );
                continue;
            }
            Expression(_) | Return(_) | Use(_) | Declaration(_) => { /* Do nothing */ }
        }
        new_statements.push(stmt.clone());
    }
    new_statements
}

/// Trait to add destructors calls to HIR
pub trait InsertDestructors {
    /// Add destructors calls to HIR
    fn insert_destructors(&mut self, context: &mut impl Context);
}

impl InsertDestructors for hir::Module {
    fn insert_destructors(&mut self, context: &mut impl Context) {
        for func in self.iter_functions_mut() {
            func.insert_destructors(context);
        }
    }
}

impl InsertDestructors for Function {
    fn insert_destructors(&mut self, context: &mut impl Context) {
        let body = with_destructors(&self.read().unwrap().body, context);
        self.write().unwrap().body = body;
    }
}
