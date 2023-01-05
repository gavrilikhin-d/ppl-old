mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod call;
pub use call::*;

use derive_more::{From, TryInto};

use crate::hir::{Type, Typed};
use crate::mutability::Mutable;
use crate::syntax::Ranged;

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    Call(Call),
}

impl Ranged for Expression {
    /// Get range of expression
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Expression::Literal(literal) => literal.range(),
            Expression::VariableReference(variable) => variable.range(),
            Expression::Call(call) => call.range(),
        }
    }
}

impl Typed for Expression {
    /// Get type of expression
    fn ty(&self) -> Type {
        match self {
            Expression::Literal(literal) => literal.ty(),
            Expression::VariableReference(variable) => variable.ty(),
            Expression::Call(call) => call.ty(),
        }
    }
}

impl Mutable for Expression {
    /// Is result of expression mutable?
    fn is_mutable(&self) -> bool {
        match self {
            Expression::Literal(_) => false,
            Expression::VariableReference(variable) => variable.is_mutable(),
            Expression::Call(call) => call.is_mutable(),
        }
    }
}
