use crate::{hir, mir::basic_block::BasicBlockData};

use super::{basic_block::Terminator, body::Body, constant::Constant, statement::Statement};

/// Trait to lower to MIR
pub trait ToMIR<C> {
    /// Resulting MIR type
    type MIR;

    /// Lower this to MIR
    fn to_mir(&self, context: C) -> Self::MIR;
}

impl ToMIR<()> for hir::Literal {
    type MIR = Constant;

    fn to_mir(&self, _: ()) -> Self::MIR {
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

impl ToMIR<&mut Body> for hir::Loop {
    type MIR = ();

    fn to_mir(&self, body: &mut Body) -> Self::MIR {
        let mut builder = BasicBlockData::build();
        self.body
            .iter()
            .map(|stmt| stmt.to_mir(body))
            .for_each(|stmt| {
                builder.add_statement(stmt);
            });
        let target = body.new_bb_id();
        let bb = builder.terminate(Terminator::GoTo { target });
        body.basic_blocks.push(bb);
    }
}

impl ToMIR<&mut Body> for hir::Statement {
    type MIR = Statement;

    fn to_mir(&self, body: &mut Body) -> Self::MIR {
        todo!()
    }
}
