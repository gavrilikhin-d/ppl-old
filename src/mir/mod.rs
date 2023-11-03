use std::sync::{Arc, Weak};

use derive_more::{From, TryInto};

use crate::hir;

/// Scope is a set of parameters and a block tree
///
/// Program is a single directional graph of scopes
#[derive(Debug, Clone)]
pub struct Scope {
    /// Parameters of the scope
    pub parameters: Vec<Weak<Variable>>,
    /// Assignments to variables in order of execution
    ///
    /// # Note
    /// Each assignment creates a new variable.
    /// All variables are immutable
    ///
    /// # Return value
    /// Last assignment is the return value
    pub variables: Vec<Arc<Variable>>,
}

/// Switch between two values depending on a condition.
///
/// # Note
/// Values must have the same type
#[derive(Debug, Clone)]
pub struct Switch {
    /// Condition to switch
    pub condition: Weak<Variable>,
    /// Expression to assign if condition is true
    pub on_true: Expression,
    /// Expression to assign if condition is false
    pub on_false: Expression,
}

/// Variable is a named value
#[derive(Debug, Clone)]
pub struct Variable {
    /// Name of the variable
    /// `None` for temporary variables
    pub name: Option<String>,
    /// Value of the variable
    pub value: Expression,
}

/// Expression is a literal, variable reference, function call or scope call
#[derive(Debug, Clone, From, TryInto)]
pub enum Expression {
    /// Literal value
    Literal(hir::Literal),
    /// Reference to a variable
    Reference(Weak<Variable>),
    /// Call to a function or a scope
    Call(Call),
    /// Switch between two values depending on a condition.
    ///
    /// # Note
    /// Values must have the same type
    Switch(Box<Switch>),
}

/// Call to a function or a call
#[derive(Debug, Clone)]
pub struct Call {
    /// Called entity
    pub callee: Callee,
    /// Arguments to pass to the function
    pub arguments: Vec<Weak<Variable>>,
}

/// Called entity
#[derive(Debug, Clone, From, TryInto)]
pub enum Callee {
    /// Call to function
    Function(String),
    /// Call to child scope
    Scope(Arc<Scope>),
    /// Call to the parent scope
    RecursiveScope(Weak<Scope>),
}
