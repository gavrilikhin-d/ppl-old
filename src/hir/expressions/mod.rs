mod literal;
pub use literal::*;

mod variable;
pub use variable::*;

mod call;
pub use call::*;

mod r#type;
pub use r#type::*;

mod member;
pub use member::*;

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
	TypeReference(TypeReference),
	MemberReference(MemberReference),
}

impl Ranged for Expression {
    /// Get range of expression
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            Expression::Literal(literal) => literal.range(),
            Expression::VariableReference(var) => var.range(),
            Expression::Call(call) => call.range(),
			Expression::TypeReference(ty) => ty.range(),
			Expression::MemberReference(member) => member.range(),
        }
    }
}

impl Typed for Expression {
    /// Get type of expression
    fn ty(&self) -> Type {
        match self {
            Expression::Literal(literal) => literal.ty(),
            Expression::VariableReference(var) => var.ty(),
            Expression::Call(call) => call.ty(),
			Expression::TypeReference(ty) => ty.ty(),
			Expression::MemberReference(member) => member.ty(),
        }
    }
}

impl Mutable for Expression {
    /// Is result of expression mutable?
    fn is_mutable(&self) -> bool {
        match self {
            Expression::Literal(l) => l.is_mutable(),
            Expression::VariableReference(var) => var.is_mutable(),
            Expression::Call(call) => call.is_mutable(),
			Expression::TypeReference(t) => t.is_mutable(),
			Expression::MemberReference(m) => m.is_mutable(),
        }
    }
}
