use inkwell::values::AnyValue;
use pretty_assertions::assert_eq;

use crate::{
    ir::{Context, FunctionContext, ModuleContext, ToIR},
    mir::{
        basic_block::{BasicBlock, BasicBlockData, SwitchCase, Terminator},
        body::Body,
        constant::Constant,
        local::{Local, LocalData},
        operand::Operand,
        package::{Function, FunctionData, Package, ParameterData, CURRENT_PACKAGE},
        statement::{Place, Projection, RValue, Statement},
        ty::{Field, Struct, StructID, Type},
    },
};

use super::ty;

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
            BasicBlockData {
                statements: vec![],
                terminator: Terminator::GoTo {
                    target: BasicBlock(1),
                },
            },
            BasicBlockData {
                statements: vec![],
                terminator: Terminator::Return,
            },
        ],
        ret: LocalData { ty: None },
        args: vec![],
        variables: vec![LocalData { ty: Bool }, LocalData { ty: I(32) }],
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
        basic_blocks: vec![BasicBlockData {
            statements: vec![],
            terminator: Terminator::Return,
        }],
        ret: LocalData { ty: I(32) },
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
        basic_blocks: vec![BasicBlockData {
            statements: vec![Assign {
                lhs: Local::ReturnValue.into(),
                rhs: 1.into(),
            }],
            terminator: Terminator::Return,
        }],
        ret: LocalData { ty: I(32) },
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
            BasicBlockData {
                statements: vec![Assign {
                    lhs: Local::ArgOrVariable(0).into(),
                    rhs: 1.into(),
                }],
                terminator: Terminator::Switch {
                    operand: Operand::Move(Local::ArgOrVariable(0)),
                    cases: vec![SwitchCase {
                        value: 3.into(),
                        target: BasicBlock(2),
                    }],
                    default: BasicBlock(1),
                },
            },
            BasicBlockData {
                statements: vec![Assign {
                    lhs: Local::ReturnValue.into(),
                    rhs: false.into(),
                }],
                terminator: Terminator::GoTo {
                    target: BasicBlock(3),
                },
            },
            BasicBlockData {
                statements: vec![Assign {
                    lhs: Local::ReturnValue.into(),
                    rhs: true.into(),
                }],
                terminator: Terminator::GoTo {
                    target: BasicBlock(3),
                },
            },
            BasicBlockData {
                statements: vec![],
                terminator: Terminator::Return,
            },
        ],
        ret: LocalData { ty: Bool },
        args: vec![],
        variables: vec![LocalData { ty: I(32) }],
    };

    let edges: Vec<_> = body.edges().collect();
    assert_eq!(
        edges,
        vec![(0, 2).into(), (0, 1).into(), (1, 3).into(), (2, 3).into()]
    );

    let f = body.to_ir(&mut context);
    let ir = f.print_to_string().to_string();
    let expected = include_str!("switch.ll");
    assert_eq!(ir, expected);
}

#[test]
fn test_struct() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test = context.module.add_function(
        "test",
        context.llvm().i32_type().fn_type(&[], false),
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    CURRENT_PACKAGE.set(Package {
        types: vec![
            ty::Struct {
                name: "Test".to_string(),
                fields: vec![Field {
                    name: "x".to_string(),
                    ty: StructID(1).into(),
                }],
            },
            ty::Struct {
                name: "Inner".to_string(),
                fields: vec![Field {
                    name: "y".to_string(),
                    ty: I(32),
                }],
            },
        ],
        functions: vec![],
    });

    use Statement::*;
    use Type::*;
    let body = Body {
        basic_blocks: vec![BasicBlockData {
            statements: vec![Assign {
                lhs: Place {
                    local: Local::ArgOrVariable(0),
                    projections: vec![
                        Projection::Field {
                            index: 0,
                            ty: StructID(1).into(),
                        },
                        Projection::Field {
                            index: 0,
                            ty: I(32),
                        },
                    ],
                },
                rhs: 1.into(),
            }],
            terminator: Terminator::Return,
        }],
        ret: LocalData { ty: I(32) },
        args: vec![],
        variables: vec![LocalData {
            ty: StructID(0).into(),
        }],
    };

    let f = body.to_ir(&mut context);
    let ir = context.module_context.module.print_to_string().to_string();
    let expected = include_str!("struct.ll");
    assert_eq!(ir, expected);
}

#[test]
fn call() {
    let llvm = inkwell::context::Context::create();
    let module = llvm.create_module("test");
    let mut context = ModuleContext::new(module);

    let test = context.module.add_function(
        "call",
        context.llvm().i32_type().fn_type(&[], false),
        Option::None,
    );
    let mut context = FunctionContext::new(&mut context, test);

    CURRENT_PACKAGE.with_borrow_mut(|package| {
        package.functions.push(FunctionData {
            name: "sum".to_string(),
            parameters: vec![
                ParameterData {
                    name: "a".to_string(),
                    ty: I(32),
                },
                ParameterData {
                    name: "b".to_string(),
                    ty: I(32),
                },
            ],
            return_type: I(32),
            body: Option::None,
        })
    });

    use Statement::*;
    use Type::*;
    let body = Body {
        basic_blocks: vec![BasicBlockData {
            statements: vec![Assign {
                lhs: Local::ReturnValue.into(),
                rhs: RValue::Call {
                    function: Function(0),
                    args: vec![1.into(), 2.into()],
                },
            }],
            terminator: Terminator::Return,
        }],
        ret: LocalData { ty: I(32) },
        args: vec![],
        variables: vec![],
    };

    let f = body.to_ir(&mut context);

    context.module_context.module.strip_debug_info();
    let ir = context.module_context.module.print_to_string().to_string();
    let expected = include_str!("call.ll");
    assert_eq!(ir, expected);
}
