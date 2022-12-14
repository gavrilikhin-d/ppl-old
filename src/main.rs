use std::io::Write;

use inkwell::execution_engine::ExecutionEngine;
use ppl::semantics::{ASTLoweringContext, ASTLoweringWithinContext, hir, Typed, Type};
use ppl::syntax::Lexer;
use ppl::syntax::ast::*;
use ppl::ir::{self, Context};
use ppl::ir::GlobalHIRLowering;
use inkwell::OptimizationLevel;

extern crate runtime;

/// Parse and compile single statement
fn process_single_statement<'llvm>(
	lexer: &mut Lexer,
	ast_lowering_context: &mut ASTLoweringContext,
	llvm: &'llvm inkwell::context::Context,
	engine: &mut ExecutionEngine<'llvm>
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

	engine.add_global_mapping(
		&context.functions().none(),
		runtime::none as usize
	);
	engine.add_global_mapping(
		&context.functions().integer_from_i64(),
		runtime::integer_from_i64 as usize
	);
	engine.add_global_mapping(
		&context.functions().integer_from_c_string(), runtime::integer_from_c_string as usize
	);

	if let Some(f) = module.get_function("initialize") {
		unsafe { engine.run_function(f, &[]); }
	}

	if let Some(f) = module.get_function("execute") {
		unsafe { engine.run_function(f, &[]); }
	}
	else if let Some(f) = module.get_function("evaluate") {
		let result = unsafe { engine.run_function(f, &[]) };
		let expr: hir::Expression = hir.try_into().unwrap();
		match expr.get_type() {
			Type::Integer => {
				let result = unsafe {
					result.into_pointer::<rug::Integer>()
				};
				println!("{}", unsafe { &*result });
			},
			Type::None => println!("none")
		}
	}

	Ok(())
}


/// Read-Evaluate-Print Loop
fn repl() {
	let mut source = String::new();
	let mut context = ASTLoweringContext::new();
	let llvm = inkwell::context::Context::create();
	let mut engine =
		llvm
			.create_module("")
			.create_jit_execution_engine(OptimizationLevel::None)
			.unwrap();

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
			&mut context,
			&llvm,
			&mut engine
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
