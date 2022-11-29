use std::fmt::Display;

use rug;

use crate::syntax::ast::Literal;

/// Evaluator for PPL
pub struct Evaluator {

}

/// Value, that may be produced by the evaluator
#[derive(Debug, PartialEq)]
pub enum Value {
	None,
	Integer(rug::Integer),
}


impl Value {
	/// Is the value a none value?
	///
	/// # Example
	/// ```
	/// use ppl::evaluator::Value;
	///
	/// let value = Value::None;
	/// assert!(value.is_none());
	///
	/// let value = Value::Integer(rug::Integer::from(42));
	/// assert!(!value.is_none());
	/// ```
	pub fn is_none(&self) -> bool {
		match self {
			Value::None => true,
			_ => false,
		}
	}
}

impl From<rug::Integer> for Value {
	fn from(value: rug::Integer) -> Self {
		Value::Integer(value)
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::None => write!(f, "none"),
			Value::Integer(value) => write!(f, "{}", value),
		}
	}
}

impl Evaluator {
	/// Evaluate value for literal
	///
	/// # Example
	/// ```
	/// use ppl::Evaluator;
	/// use ppl::syntax::ast::Literal;
	///
	/// let evaluator = Evaluator {};
	/// let literal = "none".parse::<Literal>().unwrap();
	/// let value = evaluator.evaluate_literal(&literal);
	///	assert!(value.is_none());
	/// ```
	pub fn evaluate_literal(&self, literal: &Literal) -> Value {
		match literal {
			Literal::None { offset: _ } => Value::None,
			Literal::Integer { offset: _, value } => Value::Integer(value.parse::<rug::Integer>().unwrap()),
		}
	}
}