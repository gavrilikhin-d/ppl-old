mod literal;
use enum_dispatch::enum_dispatch;
pub use literal::*;

mod variable;
pub use variable::*;

mod call;
pub use call::*;

mod r#type;
pub use r#type::*;

mod member;
pub use member::*;

mod constructor;
pub use constructor::*;

use crate::{syntax::Ranged, mutability::Mutable};

use super::Typed;

/// Any PPL expression
#[enum_dispatch(Ranged, Mutable, Typed)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    Call(Call),
	TypeReference(TypeReference),
	MemberReference(MemberReference),
	Constructor(Constructor),
}