use inkwell::values::AnyValue;
use pretty_assertions::assert_eq;

use crate::{
    ir::{Context, FunctionContext, ModuleContext, ToIR},
    mir::{
        basic_block::{BasicBlock, BasicBlockID, Terminator},
        body::Body,
        local::Local,
    },
};

#[test]
fn test_body() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test =
        context
            .module
            .add_function("test", context.llvm().void_type().fn_type(&[], false), None);
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
    let expected = include_str!("test.ll");
    assert_eq!(ir, expected);
}
