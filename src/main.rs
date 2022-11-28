use std::io::Write;

use logos::Logos;

mod syntax;
mod evaluator;
use evaluator::Evaluator;

/// Read-Evaluate-Print Loop
fn repl() {
	let evaluator = Evaluator {};

	loop {
		print!(">>> ");
		std::io::stdout().flush().unwrap();

		let mut input = String::new();
		std::io::stdin().read_line(&mut input).unwrap();

		let mut lexer = syntax::Token::lexer(&input);
		let _ = lexer.next().unwrap();
		evaluator.print_value(evaluator.evaluate_integer(lexer.slice()))
	}
}

fn main() {
	repl()
}
