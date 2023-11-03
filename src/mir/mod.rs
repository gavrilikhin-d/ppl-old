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
    /// Entry block
    pub entry: Arc<Block>,
}

/// Block is a sequence of assignments,
/// followed by a branch to another block or out of scope
#[derive(Debug, Clone)]
pub struct Block {
    /// Assignments in order of execution
    /// Each assignment creates a new variable
    pub assignments: Vec<Arc<Variable>>,
    /// Branch to another block or out of scope
    pub branch: Branch,
}

/// Branch instruction that connect blocks
#[derive(Debug, Clone, From, TryInto)]
pub enum Branch {
    /// Return from the scope
    Return(Return),
    /// Go to another branch
    GoTo(GoTo),
    /// Switch between two branches by condition
    GoToSwitch(GoToSwitch),
}

/// Return from a scope
#[derive(Debug, Clone)]
pub struct Return {
    /// Value to return
    pub value: Option<Weak<Variable>>,
}

/// Unconditionally go to another block
#[derive(Debug, Clone, From, TryInto)]
pub enum GoTo {
    /// Go to a new block
    Forward(Arc<Block>),
    /// Go to one of the previous blocks
    Backward(Weak<Block>),
}

/// Go to one of two blocks by condition
#[derive(Debug, Clone)]
pub struct GoToSwitch {
    /// Condition to switch
    pub condition: Weak<Variable>,
    /// Block to go if condition is true
    pub on_true: GoTo,
    /// Block to go if condition is false
    pub on_false: GoTo,
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
    /// Call to a function
    Call(Call),
    /// Call to a scope
    Scope(Scope),
}

/// Call to a function
#[derive(Debug, Clone)]
pub struct Call {
    /// Function to call
    pub function: String,
    /// Arguments to pass to the function
    pub arguments: Vec<Weak<Variable>>,
}
