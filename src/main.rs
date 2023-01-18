#![feature(anonymous_lifetime_in_impl_trait)]

use inkwell::execution_engine::ExecutionEngine;
use inkwell::OptimizationLevel;
use ppl::ast::*;
use ppl::hir::{self, Type, Typed};
use ppl::ir::GlobalHIRLowering;
use ppl::ir::{self, HIRModuleLowering};
use ppl::named::Named;
use ppl::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use ppl::syntax::{InteractiveLexer, Lexer, Parse};

extern crate runtime;

/// Parse and compile single statement
fn process_single_statement<'llvm>(
    parse_context: &mut ppl::syntax::Context<impl Lexer>,
    ast_lowering_context: &mut ASTLoweringContext,
    llvm: &'llvm inkwell::context::Context,
    engine: &mut ExecutionEngine<'llvm>,
) -> miette::Result<()> {
    let ast = Statement::parse(parse_context)?;
	dbg!(&ast);

    let hir = ast.lower_to_hir_within_context(ast_lowering_context)?;
    dbg!(&hir);

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
            Type::Class(c) => {
				if !c.is_builtin() {
					println!("<object of type \"{}\">", c.name())
				}
				else if c.is_integer() {
					let result = unsafe { result.into_pointer::<rug::Integer>() };
					println!("{}", unsafe { &*result });
				}
			    else if c.is_string() {
					let result = unsafe { result.into_pointer::<String>() };
					println!("{:?}", unsafe { &*result });
				}
				else if c.is_bool() {
					let result = result.as_int(false);
					if result == 0 {
						println!("false");
					}
					else {
						println!("true");
					}
				}
				else if !c.is_none()
				{
					unreachable!("forgot to handle builtin class");
				}
			},
            Type::Function { .. } => unimplemented!("returning functions"),
        }
    }

    Ok(())
}

/// Read-Evaluate-Print Loop
fn repl() {
    let mut ast_context = ASTLoweringContext::new(
		hir::Module::new("repl", "")
	);
    let llvm = inkwell::context::Context::create();
    let builtin = hir::Module::builtin().lower_to_ir(&llvm);
	builtin.print_to_stderr();
    let mut engine = builtin
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();

    let functions = ir::Functions::new(&builtin);

	/// Macro to add global mapping
	macro_rules! add_global_mapping {
		($name:ident) => {
			engine.add_global_mapping(
				&functions.$name(),
				runtime::$name as usize
			);
		};
	}

	add_global_mapping!(none);
    add_global_mapping!(integer_from_i64);
	add_global_mapping!(integer_from_c_string);
	add_global_mapping!(string_from_c_string_and_length);
	add_global_mapping!(integer_as_string);
	add_global_mapping!(print_string);
	add_global_mapping!(minus_integer);
	add_global_mapping!(integer_plus_integer);
	add_global_mapping!(integer_star_integer);


    let mut parse_context = ppl::syntax::Context::new(
		InteractiveLexer::new()
	);
    loop {
		parse_context.lexer.override_next_prompt(">>> ");

        if let Err(err) = process_single_statement(
			&mut parse_context,
			&mut ast_context,
			&llvm,
			&mut engine
		) {
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
    repl()
}
