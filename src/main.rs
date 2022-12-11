use std::io::Write;

use ppl::semantics::{ASTLoweringContext, ASTLoweringWithinContext};
use ppl::syntax::Lexer;
use ppl::syntax::ast::*;

/// Parse and compile single statement
fn process_single_statement(
	lexer: &mut Lexer,
	context: &mut ASTLoweringContext
) -> miette::Result<()> {
	let ast = Statement::parse(lexer)?;
	let hir = ast.lower_to_hir_within_context(context)?;

	println!("{:#?}", hir);

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
