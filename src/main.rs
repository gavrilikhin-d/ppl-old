#![feature(anonymous_lifetime_in_impl_trait)]

use std::cell::Cell;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use inkwell::OptimizationLevel;
use log::debug;
use ppl::ast::*;
use ppl::compilation::Compiler;
use ppl::driver::{self, Execute};
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
    llvm: &inkwell::context::Context,
    engine: &mut inkwell::execution_engine::ExecutionEngine<'llvm>,
) -> miette::Result<()> {
    let ast = Statement::parse(parse_context)?;
    debug!(target: "ast", "{:#?}", ast);

    let hir = ast.lower_to_hir_within_context(ast_lowering_context)?;
    debug!(target: "hir", "{:#?}", hir);

    let module = llvm.create_module("main");
    let mut context = ir::ModuleContext::new(module);
    hir.lower_global_to_ir(&mut context);

    let module = &context.module;

    debug!(target: "ir", "{}", module.to_string());

    module.verify().unwrap();

    engine.add_module(module).unwrap();

    if let Some(f) = module.get_function("initialize") {
        unsafe {
            engine.run_function(f, &[]);
        }
    }

    if let Some(f) = module.get_function("execute") {
        let result = unsafe { engine.run_function(f, &[]) };
        if let hir::Statement::Expression(expr) = hir {
            let ty = expr.ty().specialized();
            match ty {
                Type::Class(c) => {
                    if !c.is_builtin() {
                        // TODO: implement proper printing for user-defined classes through `as String`
                        println!("<object of type {:?}>", c.name())
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
                Type::SelfType(_) => {
                    unreachable!("Self may not be returned as result of expression")
                }
                Type::Generic(_) => unreachable!("generic types may not be returned"),
                Type::Specialized(_) => unreachable!("should be most specialized"),
            }
        }
    }

    Ok(())
}

/// Read-Evaluate-Print Loop
fn repl() {
    let mut compiler = Compiler::new();
    let mut ast_context = ModuleContext {
        module: hir::Module::new("repl", ""),
        compiler: &mut compiler,
    };

    let llvm = inkwell::context::Context::create();
    /* TODO: settings (Optimization, etc) */
    let mut engine = llvm
        .create_module("")
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    // TODO: env var for runtime path
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib_path = manifest_dir
        .join("target/debug/deps")
        .join("libruntime.dylib")
        .to_str()
        .unwrap()
        .to_string();
    let error = inkwell::support::load_library_permanently(&lib_path);
    assert!(!error, "Failed to load runtime library at: {}", &lib_path);

    let prompt = Cell::new(Some(">>> "));
    let get_line = || -> String {
        let mut content = String::new();
        loop {
            let is_first_line = prompt.get().is_some();

            print!("{}", prompt.take().unwrap_or("... "));
            std::io::stdout().lock().flush().unwrap();

            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();

            content.push_str(&line);
            if is_first_line && line.trim().is_empty() {
                prompt.set(Some(">>> "));
                continue;
            }

            return content;
        }
    };

    let mut parse_context = ppl::syntax::Context::new(InteractiveLexer::new(get_line));
    loop {
        if let Err(err) =
            process_single_statement(&mut parse_context, &mut ast_context, &llvm, &mut engine)
        {
            println!(
                "{:?}",
                err.with_source_code(miette::NamedSource::new(
                    "stdin",
                    String::from(parse_context.lexer.source())
                ))
            );
            parse_context.lexer.go_to_end();
        }

        prompt.set(Some(">>> "));
    }
}

fn main() -> miette::Result<()> {
    miette::set_panic_hook();
    pretty_env_logger::init();

    let args = driver::Args::parse();
    if let Some(cmd) = args.command {
        cmd.execute()
    } else {
        repl();
        Ok(())
    }
}
