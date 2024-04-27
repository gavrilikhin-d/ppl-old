use log::trace;

use crate::{
    hir::{
        self, Call, Expression, FunctionData, ParameterOrVariable, Statement, Typed,
        VariableReference,
    },
    syntax::Ranged,
};

use super::Context;

/// Insert destructors calls to HIR
fn with_destructors(
    statements: &[Statement],
    mut kill: Vec<ParameterOrVariable>,
    context: &mut impl Context,
) -> Vec<Statement> {
    let mut decls: Vec<ParameterOrVariable> = vec![];
    let mut new_statements = vec![];

    fn destroy(statements: &mut Vec<Statement>, v: Expression, context: &mut impl Context) {
        if let Some(destructor) = context.destructor_for(v.ty()) {
            statements.push(
                hir::Expression::from(Call {
                    range: v.range(),
                    function: destructor,
                    generic: None,
                    // FIXME: create temporary variable,
                    // if it's complex expr
                    args: vec![v],
                })
                .into(),
            );
        }
    }

    let mut stmts = vec![];
    for stmt in statements {
        match stmt {
            Statement::Block(b) => {
                stmts.extend(b.statements.iter().map(|stmt| stmt));
            }
            _ => {
                stmts.push(stmt);
            }
        }
    }
    for stmt in stmts {
        use Statement::*;
        match stmt {
            Block(_) => {
                unreachable!("Block should be flattened")
            }
            Assignment(a) => {
                destroy(&mut new_statements, a.target.clone(), context);
                new_statements.push(stmt.clone());
            }
            If(if_stmt) => {
                new_statements.push(
                    hir::If {
                        body: with_destructors(&if_stmt.body, kill.clone(), context),
                        else_block: if_stmt.else_block.as_ref().map(|else_block| hir::Else {
                            keyword: else_block.keyword.clone(),
                            body: with_destructors(&else_block.body, kill.clone(), context),
                        }),
                        else_ifs: if_stmt
                            .else_ifs
                            .iter()
                            .map(|else_if| hir::ElseIf {
                                body: with_destructors(&else_if.body, kill.clone(), context),
                                ..else_if.clone()
                            })
                            .collect(),
                        ..if_stmt.clone()
                    }
                    .into(),
                );
            }
            Loop(l) => {
                new_statements.push(
                    hir::Loop {
                        keyword: l.keyword.clone(),
                        body: with_destructors(&l.body, kill.clone(), context),
                    }
                    .into(),
                );
            }
            While(w) => {
                new_statements.push(
                    hir::While {
                        body: with_destructors(&w.body, kill.clone(), context),
                        ..w.clone()
                    }
                    .into(),
                );
            }
            Declaration(hir::Declaration::Variable(v)) => {
                kill.push(v.clone().into());
                decls.push(v.clone().into());
                new_statements.push(stmt.clone());
            }
            Declaration(hir::Declaration::Function(f)) => {
                f.write().unwrap().insert_destructors(context);
                new_statements.push(stmt.clone());
            }
            Return(ret) => {
                if let Some(hir::Expression::VariableReference(VariableReference {
                    variable,
                    ..
                })) = ret.value()
                {
                    kill.retain(|decl| decl != variable);
                    decls.retain(|decl| decl != variable);
                }
                for variable in kill {
                    let span = variable.range();
                    destroy(
                        &mut new_statements,
                        VariableReference { variable, span }.into(),
                        context,
                    );
                }
                decls = vec![];
                new_statements.push(stmt.clone());
                break;
            }
            Expression(_) | Use(_) | Declaration(_) => {
                new_statements.push(stmt.clone());
            }
        }
    }
    for v in decls {
        let span = v.range();
        let variable = v.into();
        destroy(
            &mut new_statements,
            VariableReference { variable, span }.into(),
            context,
        );
    }
    new_statements
}

/// Trait to add destructors calls to HIR
pub trait InsertDestructors {
    /// Add destructors calls to HIR
    fn insert_destructors(&mut self, context: &mut impl Context);
}

impl InsertDestructors for hir::ModuleData {
    fn insert_destructors(&mut self, context: &mut impl Context) {
        let kill = vec![];
        self.statements = with_destructors(&self.statements, kill, context);
    }
}

impl InsertDestructors for FunctionData {
    fn insert_destructors(&mut self, context: &mut impl Context) {
        if !self.is_definition() {
            return;
        }

        trace!(target: "steps", "Inserting destructors in: {self}");

        let kill = self.parameters().map(Into::into).collect();
        self.body = with_destructors(&self.body, kill, context);

        trace!(target: "steps", "After inserting destructors: {self}");
    }
}
