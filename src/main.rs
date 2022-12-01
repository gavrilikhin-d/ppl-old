use std::io::Write;

use ppl::{*, syntax::ast::Literal};


/// Read-Evaluate-Print Loop
fn repl() {
	let evaluator = Evaluator {};

	loop {
		print!(">>> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		let input = input.trim();

		let ast = input.parse::<Literal>().unwrap();
		let value = evaluator.evaluate_literal(&ast);

		if !value.is_none() {
			println!("{}", value);
		}
	}
}

fn main() {
	repl()
}
