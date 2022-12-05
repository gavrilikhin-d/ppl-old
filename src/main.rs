use std::io::Write;

use ppl::{*, syntax::ast::Statement};

fn process_single_statement(input: &str, evaluator: &mut Evaluator) -> miette::Result<()> {
	let ast = input.parse::<Statement>()?;
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
	let mut evaluator = Evaluator::new();

	loop {
		print!(">>> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		if input.trim().is_empty() {
			continue;
		}

		if let Err(err) = process_single_statement(&input, &mut evaluator) {
			println!("{:?}", err.with_source_code(input.to_string()))
		}
	}
}

fn main() {
	repl()
}
