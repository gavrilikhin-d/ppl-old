#![feature(anonymous_lifetime_in_impl_trait)]

use log::debug;
use ppl::ast::*;
use ppl::compilation::Compiler;
use ppl::hir::{self, Type, Typed};
use ppl::ir;
use ppl::ir::GlobalHIRLowering;
use ppl::named::Named;
use ppl::semantics::{ASTLoweringWithinContext, ModuleContext};
use ppl::syntax::{InteractiveLexer, Lexer, Parse};

extern crate runtime;

/// Parse and compile single statement
fn process_single_statement<'llvm>(
    parse_context: &mut ppl::syntax::Context<impl Lexer>,
    ast_lowering_context: &mut ModuleContext,
    compiler: &mut Compiler<'llvm>,
) -> miette::Result<()> {
    let ast = Statement::parse(parse_context)?;
    debug!(target: "ast", "{:#?}", ast);

    let hir = ast.lower_to_hir_within_context(ast_lowering_context)?;
    debug!(target: "hir", "{:#?}", hir);

    let module = compiler.llvm.create_module("main");
    let mut context = ir::ModuleContext::new(module);
    hir.lower_global_to_ir(&mut context);

    let module = &context.module;

    debug!(target: "ir", "{}", module.to_string());

    module.verify().unwrap();

    compiler.engine.add_module(module).unwrap();

    if let Some(f) = module.get_function("initialize") {
        unsafe {
            compiler.engine.run_function(f, &[]);
        }
    }

    if let Some(f) = module.get_function("execute") {
        unsafe {
            compiler.engine.run_function(f, &[]);
        }
    } else if let Some(f) = module.get_function("evaluate") {
        let result = unsafe { compiler.engine.run_function(f, &[]) };
        let expr: hir::Expression = hir.try_into().unwrap();
        match expr.ty() {
            Type::Class(c) => {
                if !c.is_builtin() {
                    println!("<object of type \"{}\">", c.name())
                } else if c.is_integer() {
                    let result = unsafe { result.into_pointer::<rug::Integer>() };
                    println!("{}", unsafe { &*result });
                } else if c.is_rational() {
                    let result = unsafe { result.into_pointer::<rug::Rational>() };
                    println!("{}", unsafe { &*result });
                } else if c.is_string() {
                    let result = unsafe { result.into_pointer::<String>() };
                    println!("{:?}", unsafe { &*result });
                } else if c.is_bool() {
                    let result = result.as_int(false);
                    if result == 0 {
                        println!("false");
                    } else {
                        println!("true");
                    }
                } else if !c.is_none() {
                    unreachable!("forgot to handle builtin class");
                }
            }
            Type::Function(_) => unimplemented!("returning functions"),
            Type::Trait(_) => unimplemented!("returning traits"),
            Type::SelfType(_) => unreachable!("Self may not be returned as result of expression"),
        }
    }

    Ok(())
}

/// Read-Evaluate-Print Loop
fn repl() {
    let mut ast_context = ModuleContext {
        module: hir::Module::new("repl", ""),
    };

    let llvm = inkwell::context::Context::create();
    let mut compiler = Compiler::new(&llvm);

    let mut parse_context = ppl::syntax::Context::new(InteractiveLexer::new());
    loop {
        parse_context.lexer.override_next_prompt(">>> ");

        if let Err(err) =
            process_single_statement(&mut parse_context, &mut ast_context, &mut compiler)
        {
            println!(
                "{:?}",
                err.with_source_code(miette::NamedSource::new(
                    "stdin",
                    String::from(parse_context.lexer.source())
                ))
            )
        }
    }
}

fn main() {
    miette::set_panic_hook();
    pretty_env_logger::init();
    repl()
}
