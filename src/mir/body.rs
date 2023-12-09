use crate::ir::{Context, FunctionContext, ToIR};

use super::{basic_block::BasicBlock, local::Local};

pub struct Body {
    pub basic_blocks: Vec<BasicBlock>,
    pub locals: Vec<Local>,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Body {
    type IR = inkwell::values::FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        for (i, _) in self.locals.iter().enumerate() {
            let ty = context.types().i64();
            let name = format!("_{i}");
            context.builder.build_alloca(ty, &name);
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

#[cfg(test)]
mod test {
    use inkwell::values::AnyValue;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        ir::ModuleContext,
        mir::{
            basic_block::{BasicBlockID, Terminator},
            local::Local,
        },
    };

    #[test]
    fn test_body() {
        let llvm = inkwell::context::Context::create();
        let module = llvm.create_module("test");
        let mut context = ModuleContext::new(module);

        let test = context.module.add_function(
            "test",
            context.llvm().void_type().fn_type(&[], false),
            None,
        );
        let mut context = FunctionContext::new(&mut context, test);

        let body = Body {
            basic_blocks: vec![
                BasicBlock {
                    terminator: Terminator::GoTo {
                        target: BasicBlockID(1),
                    },
                },
                BasicBlock {
                    terminator: Terminator::Return,
                },
            ],
            locals: vec![Local {}, Local {}, Local {}],
        };

        let f = body.to_ir(&mut context);
        let ir = f.print_to_string().to_string();
        let expected = r#"
define void @test() {
  %_0 = alloca i64, align 8
  %_1 = alloca i64, align 8
  %_2 = alloca i64, align 8
  br label %bb0

bb0:                                              ; preds = %0
  br label %bb1

bb1:                                              ; preds = %bb0
  ret void
}
"#;
        assert_eq!(ir, expected.trim_start());
    }
}
