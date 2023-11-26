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

mod implicit_conversion;
pub use implicit_conversion::*;

use crate::{mutability::Mutable, syntax::Ranged};

use derive_more::Display;

use super::Generic;

/// Any PPL expression
#[enum_dispatch(Ranged, Mutable, Typed)]
#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub enum Expression {
    Literal(Literal),
    VariableReference(VariableReference),
    Call(Call),
    TypeReference(TypeReference),
    MemberReference(MemberReference),
    Constructor(Constructor),
    ImplicitConversion(ImplicitConversion),
}

impl Expression {
    /// Check if expression is a reference to something
    pub fn is_reference(&self) -> bool {
        matches!(
            self,
            Expression::VariableReference(_)
                | Expression::MemberReference(_)
                | Expression::TypeReference(_)
        )
    }
}

impl Generic for Expression {
    fn is_generic(&self) -> bool {
        match self {
            Expression::Literal(_) => false,
            Expression::VariableReference(v) => v.is_generic(),
            Expression::Call(c) => c.is_generic(),
            Expression::TypeReference(t) => t.is_generic(),
            Expression::MemberReference(m) => m.is_generic(),
            Expression::Constructor(c) => c.is_generic(),
            Expression::ImplicitConversion(i) => i.is_generic(),
        }
    }
}
