mod assignment;
use std::fmt::Debug;

pub use assignment::*;

mod ret;
use derive_visitor::DriveMut;
pub use ret::*;

mod r#if;
pub use r#if::*;

mod r#loop;
pub use r#loop::*;

mod r#while;
pub use r#while::*;

mod r#use;
pub use r#use::*;

use derive_more::{Display, From, TryInto};

use crate::{
    hir::{Declaration, Expression},
    syntax::Ranged,
};

/// Block of statements
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct Block {
    pub statements: Vec<Statement>,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            std::fmt::Display::fmt(&statement, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Ranged for Block {
    fn start(&self) -> usize {
        self.statements.first().unwrap().start()
    }

    fn end(&self) -> usize {
        self.statements.last().unwrap().end()
    }
}

/// Any PPL statement
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto, DriveMut)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
    Return(Return),
    If(If),
    Loop(Loop),
    While(While),
    Use(Use),
    Block(Block),
}

impl Ranged for Statement {
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Statement::Declaration(declaration) => declaration.range(),
            Statement::Expression(expression) => expression.range(),
            Statement::Assignment(assignment) => assignment.range(),
            Statement::Return(r#return) => r#return.range(),
            Statement::If(r#if) => r#if.range(),
            Statement::Loop(r#loop) => r#loop.range(),
            Statement::While(r#while) => r#while.range(),
            Statement::Use(r#use) => r#use.range(),
            Statement::Block(block) => block.range(),
        }
    }
}
