use inkwell::types::BasicTypeEnum;

use crate::ir::{Context, FunctionContext, ToIR};

use super::{basic_block::BasicBlock, local::Local};

pub struct Body {
    pub basic_blocks: Vec<BasicBlock>,
    pub locals: Vec<Local>,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Body {
    type IR = inkwell::values::FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        for (i, local) in self.locals.iter().enumerate() {
            let ty = local.ty.to_ir(context);
            if let Ok(ty) = BasicTypeEnum::try_from(ty) {
                let name = format!("_{i}");
                context.builder.build_alloca(ty, &name);
            }
        }

        for i in 0..self.basic_blocks.len() {
            let name = format!("bb{i}");
            context.llvm().append_basic_block(context.function, &name);
        }

        let bb0 = context.function.get_basic_blocks().get(1).unwrap().clone();
        context.builder.build_unconditional_branch(bb0);

        for (i, block) in self.basic_blocks.iter().enumerate() {
            let bb = context
                .function
                .get_basic_blocks()
                .get(i + 1)
                .unwrap()
                .clone();
            context.builder.position_at_end(bb);
            block.to_ir(context);
        }

        context.function.clone()
    }
}
