use std::io::Write;

use ppl::{*, syntax::ast::Statement};


/// Read-Evaluate-Print Loop
fn repl() {
	let mut evaluator = Evaluator::new();

	loop {
		print!(">>> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		let input = input.trim();

		let ast = input.parse::<Statement>().unwrap();
		let value = evaluator.execute(&ast).unwrap();

		if value.is_none() { continue; }

		let value = value.unwrap();

		if !value.is_none() {
			println!("{}", value);
		}
	}
}

fn main() {
	repl()
}
