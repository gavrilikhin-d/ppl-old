use std::fmt::Display;

use rug;

use crate::syntax::ast::{Literal, Expression, Statement, Declaration, VariableDeclaration};

use crate::evaluate::error::*;

/// Value, that may be produced by the evaluator
#[derive(Debug, PartialEq, Clone)]
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

/// Data of the variable
struct VariableData {
	/// Computed value of the variable
	value: Value,
	/// Declaration of the variable
	declaration: VariableDeclaration,
}

/// Evaluator for PPL
pub struct Evaluator {
	/// Declared variables
	variables: std::collections::HashMap<String, VariableData>,
}

impl Evaluator {
	/// Create new evaluator
	///
	/// # Example
	/// ```
	/// use ppl::evaluator::Evaluator;
	///
	/// let mut evaluator = Evaluator::new();
	/// ```
	pub fn new() -> Self {
		Self {
			variables: std::collections::HashMap::new(),
		}
	}

	/// Evaluate value for literal
	///
	/// # Example
	/// ```
	/// use ppl::Evaluator;
	/// use ppl::syntax::ast::Literal;
	///
	/// let evaluator = Evaluator::new();
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

	/// Evaluate value for expression
	///
	/// # Example
	/// ```
	/// use ppl::Evaluator;
	/// use ppl::syntax::ast::Expression;
	/// use ppl::syntax::ast::Literal;
	/// use ppl::evaluator::Value;
	///
	/// let evaluator = Evaluator::new();
	/// let expression = "42".parse::<Expression>().unwrap();
	/// let value = evaluator.evaluate_expression(&expression).unwrap();
	/// assert_eq!(value, Value::Integer(42.into()));
	/// ```
	pub fn evaluate_expression(&self, expr: &Expression) -> Result<Value, UndefinedVariable> {
		Ok(
			match expr {
				Expression::Literal(l) => self.evaluate_literal(l),
				Expression::VariableReference(var) => {
					let data = self.variables.get(&var.name.value);
					if let Some(data) = data {
						data.value.clone()
					} else {
						return Err(UndefinedVariable {
							name: var.name.value.clone(),
							at: var.name.range().into()
						});
					}
				}
			}
		)
	}

	/// Execute code for declaration
	pub fn declare(&mut self, decl: &Declaration) -> Result<(), UndefinedVariable> {
		match decl {
			Declaration::Variable(var) => {
				let value = self.evaluate_expression(&var.initializer)?;
				self.variables.insert(var.name.value.clone(), VariableData {
					value,
					declaration: var.clone(),
				});
				Ok(())
			}
		}
	}

	/// Execute statement
	pub fn execute(&mut self, stmt: &Statement) -> Result<Option<Value>, Error> {
		Ok(match stmt {
			Statement::Expression(expr) =>
				Some(self.evaluate_expression(expr)?),
			Statement::Declaration(decl) => {
				self.declare(decl)?;
				None
			},
			Statement::Assignment(a) => {
				match &a.target {
					Expression::VariableReference(var) => {
						if self.variables.get(&var.name.value).is_none() {
							return Err(UndefinedVariable {
								name: var.name.value.clone(),
								at: var.name.range().into()
							}.into());
						}

						let decl = &self.variables.get(&var.name.value).unwrap().declaration;

						if decl.is_immutable() {
							return Err(AssignmentToImmutable {
								name: var.name.value.clone(),
								referenced_at: var.name.range().into(),
								declared_at: decl.name.range().into()
							}.into());
						}

						let value = self.evaluate_expression(&a.value)?;
						self.variables.get_mut(&var.name.value).unwrap().value = value;
						None
					},
					Expression::Literal(_) => {
						unimplemented!("error: cannot assign to literal");
					}
				}
			}
		})
	}
}

impl Default for Evaluator {
	/// Create new evaluator without variables
	fn default() -> Self {
		Self::new()
	}
}