#![feature(anonymous_lifetime_in_impl_trait)]

use std::cell::Cell;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use inkwell::OptimizationLevel;
use log::debug;
use miette::NamedSource;
use ppl::compilation::Compiler;
use ppl::driver::commands::compile::OutputType;
use ppl::driver::{self, Execute};
use ppl::hir;
use ppl::ir::HIRModuleLowering;
use ppl::semantics::{ModuleContext, Monomorphize, ToHIR};
use ppl::syntax::{InteractiveLexer, Lexer, Parse};
use ppl::Reporter;
use ppl::{ast::*, SourceFile};

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

    let mut hir = ast.to_hir(ast_lowering_context)?;
    hir.monomorphize(ast_lowering_context);
    debug!(target: "hir", "{:#}", hir);

    ast_lowering_context.module.statements = vec![hir];

    let with_main = true;
    let module = ast_lowering_context.module.to_ir(llvm, with_main);
    debug!(target: "ir", "{}", module.to_string());

    module.verify().unwrap();

    engine.add_module(&module).unwrap();

    if let Some(f) = module.get_function("main") {
        unsafe { engine.run_function_as_main(f, &[]) };
    }

    Ok(())
}

/// Read-Evaluate-Print Loop
fn repl() {
    let mut compiler = Compiler::new();
    let mut ast_context = ModuleContext {
        module: hir::ModuleData::new(SourceFile::in_memory(NamedSource::new(
            "repl",
            "".to_string(),
        ))),
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
        .join(OutputType::DynamicLibrary.named("ppl"));
    inkwell::support::load_library_permanently(&lib_path).expect(&format!(
        "Failed to load core library at: {}",
        lib_path.display()
    ));

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
    miette::set_hook(Box::new(|_| Box::new(Reporter::default())))?;
    pretty_env_logger::init();

    let args = driver::Args::parse();
    if let Some(cmd) = args.command {
        cmd.execute()
    } else {
        repl();
        Ok(())
    }
}
