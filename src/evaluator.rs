use rug;

/// Evaluator for PPL
pub struct Evaluator {

}

/// Value, that may be produced by the evaluator
pub enum Value {
	Integer(rug::Integer),
}

impl From<rug::Integer> for Value {
	fn from(value: rug::Integer) -> Self {
		Value::Integer(value)
	}
}

impl Evaluator {
	/// Evaluate value for integer literal
	pub fn evaluate_integer(&self, value: &str) -> rug::Integer {
		rug::Integer::from_str_radix(value, 10).unwrap()
	}

	/// Print value received from the evaluator
	pub fn print_value<V: Into<Value>>(&self, value: V) {
		match value.into() {
			Value::Integer(value) => println!("{}", value),
		}
	}
}

#[test]
fn test_printing() {
	let evaluator = Evaluator {};

	evaluator.print_value(evaluator.evaluate_integer("123"));
}