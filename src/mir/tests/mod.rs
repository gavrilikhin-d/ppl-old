use inkwell::values::AnyValue;
use pretty_assertions::assert_eq;

use crate::{
    ir::{Context, FunctionContext, ModuleContext, ToIR},
    mir::{
        basic_block::{BasicBlock, BasicBlockID, SwitchCase, Terminator},
        body::Body,
        constant::Constant,
        local::{Local, LocalID},
        operand::Operand,
        statement::Statement,
        ty::Type,
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
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    use Type::*;
    let body = Body {
        basic_blocks: vec![
            BasicBlock {
                statements: vec![],
                terminator: Terminator::GoTo {
                    target: BasicBlockID(1),
                },
            },
            BasicBlock {
                statements: vec![],
                terminator: Terminator::Return,
            },
        ],
        ret: Local { ty: None },
        args: vec![],
        variables: vec![Local { ty: Bool }, Local { ty: I(32) }],
    };

    let f = body.to_ir(&mut context);
    let ir = f.print_to_string().to_string();
    let expected = include_str!("test.ll");
    assert_eq!(ir, expected);
}

#[test]
fn return_value() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test = context.module.add_function(
        "return_value",
        context.llvm().i32_type().fn_type(&[], false),
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    use Type::*;
    let body = Body {
        basic_blocks: vec![BasicBlock {
            statements: vec![],
            terminator: Terminator::Return,
        }],
        ret: Local { ty: I(32) },
        args: vec![],
        variables: vec![],
    };

    let f = body.to_ir(&mut context);
    let ir = f.print_to_string().to_string();
    let expected = include_str!("return_value.ll");
    assert_eq!(ir, expected);
}

#[test]
fn assign() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test = context.module.add_function(
        "assign",
        context.llvm().i32_type().fn_type(&[], false),
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    use Statement::*;
    use Type::*;
    let body = Body {
        basic_blocks: vec![BasicBlock {
            statements: vec![Assign {
                lhs: LocalID::FOR_RETURN_VALUE,
                rhs: Constant::i32(1).into(),
            }],
            terminator: Terminator::Return,
        }],
        ret: Local { ty: I(32) },
        args: vec![],
        variables: vec![],
    };

    let f = body.to_ir(&mut context);
    let ir = f.print_to_string().to_string();
    let expected = include_str!("assign.ll");
    assert_eq!(ir, expected);
}

#[test]
fn switch() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test = context.module.add_function(
        "switch",
        context.llvm().bool_type().fn_type(&[], false),
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    use Statement::*;
    use Type::*;
    let body = Body {
        basic_blocks: vec![
            BasicBlock {
                statements: vec![Assign {
                    lhs: LocalID(1),
                    rhs: 1.into(),
                }],
                terminator: Terminator::Switch {
                    operand: Operand::Move(LocalID(1)),
                    cases: vec![SwitchCase {
                        value: 3.into(),
                        target: BasicBlockID(2),
                    }],
                    default: BasicBlockID(1),
                },
            },
            BasicBlock {
                statements: vec![Assign {
                    lhs: LocalID::FOR_RETURN_VALUE,
                    rhs: false.into(),
                }],
                terminator: Terminator::GoTo {
                    target: BasicBlockID(3),
                },
            },
            BasicBlock {
                statements: vec![Assign {
                    lhs: LocalID::FOR_RETURN_VALUE,
                    rhs: true.into(),
                }],
                terminator: Terminator::GoTo {
                    target: BasicBlockID(3),
                },
            },
            BasicBlock {
                statements: vec![],
                terminator: Terminator::Return,
            },
        ],
        ret: Local { ty: Bool },
        args: vec![],
        variables: vec![Local { ty: I(32) }],
    };

    let edges = body.edges();
    assert_eq!(
        edges,
        vec![
            (BasicBlockID(0), BasicBlockID(2)),
            (BasicBlockID(0), BasicBlockID(1)),
            (BasicBlockID(1), BasicBlockID(3)),
            (BasicBlockID(2), BasicBlockID(3))
        ]
    );

    let f = body.to_ir(&mut context);
    let ir = f.print_to_string().to_string();
    let expected = include_str!("switch.ll");
    assert_eq!(ir, expected);
}
