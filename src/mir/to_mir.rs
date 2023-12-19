use crate::hir;

use super::constant::Constant;

/// Trait to lover to MIR
pub trait ToMIR<C> {
    /// Resulting MIR type
    type MIR;

    /// Lover this to MIR
    fn to_mir(&self, context: &mut C) -> Self::MIR;
}

impl ToMIR<()> for hir::Literal {
    type MIR = Constant;

    fn to_mir(&self, _: &mut ()) -> Self::MIR {
        use hir::Literal::*;
        match self {
            None { .. } => Constant::None,
            Bool { value, .. } => Constant::Bool(*value),

            Integer { .. } => todo!(),
            Rational { .. } => todo!(),
            String { .. } => todo!(),
        }
    }
}
