mod assignment;
pub use assignment::*;

use derive_more::{From, TryInto};

use crate::hir::{Declaration, Expression};

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
    Assignment(Assignment),
}
