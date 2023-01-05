#![feature(anonymous_lifetime_in_impl_trait)]

use inkwell::execution_engine::ExecutionEngine;
use inkwell::OptimizationLevel;
use ppl::ast::*;
use ppl::hir::{self, Type, Typed};
use ppl::ir::GlobalHIRLowering;
use ppl::ir::{self, HIRModuleLowering};
use ppl::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use ppl::syntax::{InteractiveLexer, Lexer, Parse};

extern crate runtime;

/// Parse and compile single statement
fn process_single_statement<'llvm>(
    lexer: &mut impl Lexer,
    ast_lowering_context: &mut ASTLoweringContext,
    llvm: &'llvm inkwell::context::Context,
    engine: &mut ExecutionEngine<'llvm>,
) -> miette::Result<()> {
    let ast = Statement::parse(lexer)?;
    println!("AST: {:#?}", ast);

    let hir = ast.lower_to_hir_within_context(ast_lowering_context)?;
    println!("HIR: {:#?}", hir);

    let module = llvm.create_module("main");
    let mut context = ir::ModuleContext::new(module);
    hir.lower_global_to_ir(&mut context);

    let module = &context.module;

    module.print_to_stderr();

    module.verify().unwrap();

    engine.add_module(module).unwrap();

    if let Some(f) = module.get_function("initialize") {
        unsafe {
            engine.run_function(f, &[]);
        }
    }

    if let Some(f) = module.get_function("execute") {
        unsafe {
            engine.run_function(f, &[]);
        }
    } else if let Some(f) = module.get_function("evaluate") {
        let result = unsafe { engine.run_function(f, &[]) };
        let expr: hir::Expression = hir.try_into().unwrap();
        match expr.ty() {
            Type::Integer => {
                let result = unsafe { result.into_pointer::<rug::Integer>() };
                println!("{}", unsafe { &*result });
            }
            Type::None => (), // Do nothing
            Type::String => {
                let result = unsafe { result.into_pointer::<String>() };
                println!("{:?}", unsafe { &*result });
            }
            Type::Class(_) => unimplemented!("returning classes"),
            Type::Function { .. } => unimplemented!("returning functions"),
        }
    }

    Ok(())
}

/// Read-Evaluate-Print Loop
fn repl() {
    let mut context = ASTLoweringContext::new("repl");
    let llvm = inkwell::context::Context::create();
    let builtin = hir::Module::builtin().lower_to_ir(&llvm);
    let mut engine = builtin
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    let functions = ir::Functions::new(&builtin);

    engine.add_global_mapping(&functions.none(), runtime::none as usize);
    engine.add_global_mapping(
        &functions.integer_from_i64(),
        runtime::integer_from_i64 as usize,
    );
    engine.add_global_mapping(
        &functions.integer_from_c_string(),
        runtime::integer_from_c_string as usize,
    );
    engine.add_global_mapping(
        &functions.string_from_c_string_and_length(),
        runtime::string_from_c_string_and_length as usize,
    );
    engine.add_global_mapping(&functions.print_integer(), runtime::print_integer as usize);
    engine.add_global_mapping(&functions.print_string(), runtime::print_string as usize);

    let mut lexer = InteractiveLexer::new();
    loop {
        if let Err(err) = process_single_statement(&mut lexer, &mut context, &llvm, &mut engine) {
            println!(
                "{:?}",
                err.with_source_code(miette::NamedSource::new(
                    "stdin",
                    String::from(lexer.source())
                ))
            )
        }
    }
}

fn main() {
    miette::set_panic_hook();
    repl()
}
