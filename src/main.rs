use std::io::Write;

use ppl::{*, syntax::{ast::{Statement, Parse}, Token}};

use logos::Logos;

fn process_single_statement(lexer: &mut logos::Lexer<Token>, evaluator: &mut Evaluator) -> miette::Result<()> {
	let ast = Statement::parse(lexer)?;
	let value = evaluator.execute(&ast)?;

	if value.is_none() { return Ok(()); }

	let value = value.unwrap();

	if !value.is_none() {
		println!("{}", value);
	}

	Ok(())
}


/// Read-Evaluate-Print Loop
fn repl() {
	let mut source = String::new();
	let mut evaluator = Evaluator::new();

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

		let mut lexer = Token::lexer(&source);
		lexer.bump(offset);

		if let Err(err) = process_single_statement(
			&mut lexer,
			&mut evaluator
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
