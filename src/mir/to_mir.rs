use crate::{hir, mir::basic_block::BasicBlockData};

use super::{
    basic_block::Terminator, body::Body, constant::Constant, operand::Operand, statement::Statement,
};

/// Trait to lower to MIR
pub trait ToMIR {
    /// Context required to lower to mir
    type Context;
    /// Resulting MIR type
    type MIR;

    /// Lower this to MIR
    fn to_mir(&self, context: &mut Self::Context) -> Self::MIR;
}

impl ToMIR for hir::Literal {
    type Context = Body;
    type MIR = Operand;

    fn to_mir(&self, body: &mut Body) -> Self::MIR {
        use hir::Literal::*;
        match self {
            None { .. } => Constant::None.into(),
            Bool { value, .. } => Constant::Bool(*value).into(),

            Integer { .. } => Operand::Move(body.new_allocated_constant(todo!(), todo!())),
            Rational { .. } => Operand::Move(body.new_allocated_constant(todo!(), todo!())),
            String { .. } => Operand::Move(body.new_allocated_constant(todo!(), todo!())),
        }
    }
}

impl ToMIR for hir::Loop {
    type Context = Body;
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

impl ToMIR for hir::Statement {
    type Context = Body;
    type MIR = Statement;

    fn to_mir(&self, body: &mut Body) -> Self::MIR {
        todo!()
    }
}
