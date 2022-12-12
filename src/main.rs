use std::io::Write;

use ppl::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use ppl::syntax::Lexer;
use ppl::syntax::ast::*;
use ppl::ir;
use ppl::ir::GlobalHIRLowering;
use inkwell::OptimizationLevel;

extern crate runtime;

/// Parse and compile single statement
fn process_single_statement(
	lexer: &mut Lexer,
	context: &mut ASTLoweringContext
) -> miette::Result<()> {
	let ast = Statement::parse(lexer)?;
	println!("AST: {:#?}", ast);

	let hir = ast.lower_to_hir_within_context(context)?;
	println!("HIR: {:#?}", hir);

	let llvm = inkwell::context::Context::create();
	let module = llvm.create_module("main");
	let mut context = ir::ModuleContext::new(module);
	hir.lower_global_to_ir(&mut context);

	let module = context.module;

	module.verify().unwrap();

	module.print_to_stderr();


	let engine =
		module
			.create_jit_execution_engine(OptimizationLevel::None)
			.unwrap();

	engine.add_global_mapping(&context.functions.none, runtime::none as usize);

	Ok(())
}


/// Read-Evaluate-Print Loop
fn repl() {
	let mut source = String::new();
	let mut context = ASTLoweringContext::new();

	loop {
		print!(">>> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		if input.trim().is_empty() {
			continue;
		}

		let offset = source.len();

		source.push_str(&input);

		let mut lexer = Lexer::new(&source);
		lexer.bump(offset);

		if let Err(err) = process_single_statement(
			&mut lexer,
			&mut context
		) {
			println!(
				"{:?}",
				err.with_source_code(
					miette::NamedSource::new("stdin", source.clone())
				)
			)
		}
	}
}

fn main() {
	repl()
}
