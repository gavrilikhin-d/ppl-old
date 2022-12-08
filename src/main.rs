use std::io::Write;

use ppl::{*, syntax::{ast::{Statement, Parse}, Lexer}, semantics::ast_to_hir::Lowering};

/// Parse and compile single statement
fn process_single_statement(lexer: &mut Lexer, module: &mut semantics::Module) -> miette::Result<()> {
	let ast = Statement::parse(lexer)?;
	let hir = module.add(&ast)?;

	println!("{:#?}", hir);

	Ok(())
}


/// Read-Evaluate-Print Loop
fn repl() {
	let mut source = String::new();
	let mut module = semantics::Module::new();

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
			&mut module
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
