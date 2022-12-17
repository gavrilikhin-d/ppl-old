use derive_more::{From, TryInto};

use std::fmt::Display;
use std::hash::{Hash, Hasher};

use crate::syntax::{WithOffset, Ranged};

mod module;
pub use module::*;

mod types;
pub use types::*;

pub use crate::mutability::{Mutability, Mutable};


/// AST for compile time known values
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Literal {
	/// None literal
	None { offset: usize },
	/// Any precision decimal integer literal
	Integer { span: std::ops::Range<usize>, value: rug::Integer },
}

impl Ranged for Literal {
	/// Get range of literal
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Literal::None { offset } =>
				*offset..*offset + 4,
			Literal::Integer { span, .. } =>
				span.clone(),
		}
	}
}

impl Typed for Literal {
	/// Get type of literal
	fn get_type(&self) -> Type {
		match self {
			Literal::None { .. } => Type::None,
			Literal::Integer { .. } => Type::Integer,
		}
	}
}

/// AST for variable reference
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableReference {
	/// Range of variable reference
	pub span: std::ops::Range<usize>,
	/// Referenced variable name
	pub variable: Box<VariableDeclaration>,
}

impl Mutable for VariableReference {
	/// Check if referenced variable is mutable
	fn is_mutable(&self) -> bool {
		self.variable.is_mutable()
	}
}

impl Ranged for VariableReference {
	/// Get range of variable reference
	fn range(&self) -> std::ops::Range<usize> {
		self.span.clone()
	}
}

impl Typed for VariableReference {
	/// Get type of variable reference
	fn get_type(&self) -> Type {
		self.variable.get_type()
	}
}

/// Any PPL expression
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Expression {
	Literal(Literal),
	VariableReference(VariableReference),
}

impl Ranged for Expression {
	/// Get range of expression
	fn range(&self) -> std::ops::Range<usize> {
		match self {
			Expression::Literal(literal) => literal.range(),
			Expression::VariableReference(variable) => variable.range(),
		}
	}
}

impl Typed for Expression {
	/// Get type of expression
	fn get_type(&self) -> Type {
		match self {
			Expression::Literal(literal) => literal.get_type(),
			Expression::VariableReference(variable) => variable.get_type(),
		}
	}
}

impl Mutable for Expression {
	/// Is result of expression mutable?
	fn is_mutable(&self) -> bool {
		match self {
			Expression::Literal(_) => false,
			Expression::VariableReference(variable) => variable.is_mutable(),
		}
	}
}

/// Declaration of a variable
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VariableDeclaration {
	/// Variable's name
	pub name: WithOffset<String>,
	/// Initializer for variable
	pub initializer: Expression,

	/// Mutability of variable
	pub mutability: Mutability,
}

impl Hash for VariableDeclaration {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.value.hash(state);
	}
}

impl Mutable for VariableDeclaration {
	/// Is variable declared as mutable?
	fn is_mutable(&self) -> bool {
		self.mutability.is_mutable()
	}
}

impl Typed for VariableDeclaration {
	/// Get type of variable
	fn get_type(&self) -> Type {
		self.initializer.get_type()
	}
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TypeDeclaration {
	/// Type's name
	pub name: WithOffset<String>,
}

/// Declaration of a function parameter
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameter {
	/// Type's name
	pub name: WithOffset<String>,
	/// Type of parameter
	pub ty: Type
}

impl Typed for Parameter {
	/// Get type of parameter
	fn get_type(&self) -> Type {
		self.ty.clone()
	}
}

/// Part of a function name
#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum FunctionNamePart {
	Text(WithOffset<String>),
	Parameter(Parameter),
}

impl Display for FunctionNamePart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			FunctionNamePart::Text(text) => write!(f, "{}", text.value),
			FunctionNamePart::Parameter(parameter) =>
				write!(f, "<{}: {}>", parameter.name.value, parameter.ty),
		}
	}
}

/// Declaration of a type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FunctionDeclaration {
	/// Type's name
	pub name_parts: Vec<FunctionNamePart>,
	/// Type of returned value
	pub return_type: Type,
}

impl FunctionDeclaration {
	/// Get name of function
	pub fn name(&self) -> String {
		let mut name = String::new();

		for (i, part) in self.name_parts.iter().enumerate() {
			if i > 0 {
				name.push_str(" ");
			}
			name.push_str(format!("{}", part).as_str());
		}

		name
	}
}


impl Typed for FunctionDeclaration {
	fn get_type(&self) -> Type {
		Type::Function {
			return_type: Box::new(self.return_type.clone()),
			parameters:
				self.name_parts.iter().filter_map(
					|part| match part {
						FunctionNamePart::Parameter(parameter) =>
							Some(parameter.get_type()),
						_ => None
					}
				).collect()
		}
	}
}

/// Any PPL declaration
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Declaration {
	Variable(VariableDeclaration),
	Type(TypeDeclaration),
	Function(FunctionDeclaration),
}

/// Assignment of a value to a
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Assignment {
	/// Variable to assign to
	pub target: Expression,
	/// Value to assign
	pub value: Expression,
}

/// Any PPL statement
#[derive(Debug, PartialEq, Eq, Clone, From, TryInto)]
pub enum Statement {
	Declaration(Declaration),
	Expression(Expression),
	Assignment(Assignment),
}