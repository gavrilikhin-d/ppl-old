use crate::ir::{Context, FunctionContext, ToIR};

use super::{
    basic_block::{BasicBlock, BasicBlockID},
    local::Local,
};

/// Edge from one basic block to another
#[derive(Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: BasicBlockID,
    pub to: BasicBlockID,
}

impl Edge {
    /// Get reversed edge
    pub fn reversed(self) -> Self {
        Self {
            from: self.to,
            to: self.from,
        }
    }
}

impl From<(usize, usize)> for Edge {
    fn from(value: (usize, usize)) -> Self {
        let (from, to) = value;
        Edge {
            from: BasicBlockID(from),
            to: BasicBlockID(to),
        }
    }
}

impl From<(BasicBlockID, BasicBlockID)> for Edge {
    fn from(value: (BasicBlockID, BasicBlockID)) -> Self {
        let (from, to) = value;
        Edge { from, to }
    }
}

#[derive(Clone)]
pub struct Body {
    pub basic_blocks: Vec<BasicBlock>,
    pub ret: Local,
    pub args: Vec<Local>,
    pub variables: Vec<Local>,
}

impl Body {
    pub const RETURN_VALUE_NAME: &'static str = "_0";

    pub fn locals(&self) -> impl Iterator<Item = &Local> {
        std::iter::once(&self.ret)
            .chain(self.args.iter())
            .chain(self.variables.iter())
    }

    /// Edges of CFG in this body
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.basic_blocks.iter().enumerate().flat_map(|(i, block)| {
            let from = BasicBlockID(i);
            block
                .terminator
                .destinations()
                .map(move |to| Edge { from, to })
        })
    }

    /// Get edges from specified block
    pub fn edges_from(&self, from: BasicBlockID) -> impl Iterator<Item = Edge> + '_ {
        let block = &self.basic_blocks[from.0];
        block.successors().map(move |to| Edge { from, to })
    }

    /// Get edges to specified block
    pub fn edges_to(&self, to: BasicBlockID) -> impl Iterator<Item = Edge> + '_ {
        self.edges().filter(move |e| e.to == to)
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Body {
    type IR = inkwell::values::FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        // TODO: remove cloning
        context.body = self.clone();

        for local in self.locals() {
            local.to_ir(context);
        }

        for i in 0..self.basic_blocks.len() {
            let name = format!("bb{i}");
            context.llvm().append_basic_block(context.function, &name);
        }

        let bb0 = context.bb(BasicBlockID(0));
        context.builder.build_unconditional_branch(bb0);

        for (i, block) in self.basic_blocks.iter().enumerate() {
            let bb = context.bb(BasicBlockID(i));
            context.builder.position_at_end(bb);
            block.to_ir(context);
        }

        context.function.clone()
    }
}
