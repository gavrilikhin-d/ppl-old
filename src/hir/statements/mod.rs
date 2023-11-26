mod assignment;
pub use assignment::*;

mod ret;
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

use crate::hir::{Declaration, Expression};

/// Any PPL statement
#[derive(Debug, Display, PartialEq, Eq, Clone, From, TryInto)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
    Return(Return),
    If(If),
    Loop(Loop),
    While(While),
    Use(Use),
}
