use core::fmt;
use std::fmt::{Debug, Display};

use derive_more::{DebugCustom, Display, From, TryInto};

use crate::hir;

mod hir_to_mir;
pub use hir_to_mir::*;

/// Scope is a set of parameters and a block tree
///
/// Program is a single directional graph of scopes
#[derive(Clone)]
pub struct Scope {
    /// Name of the scope
    pub name: String,
    /// Parameters of the scope
    pub parameters: Vec<VariableReference>,
    /// Assignments to variables in order of execution
    pub variables: Vec<Variable>,
    /// Returned variable
    pub ret: Option<VariableReference>,
}

impl Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = f.width().unwrap_or(0);

        write!(f, "{:indent$}", "\t")?;
        let mut signature = f.debug_tuple(&self.name);
        for param in self.parameters.iter() {
            signature.field(param);
        }
        signature.finish()?;
        writeln!(f, ":")?;

        let indent = indent + 1;
        for var in self.variables.iter() {
            writeln!(f, "{var:indent$}")?;
        }
        Ok(())
    }
}

/// Variable is a named value
#[derive(Clone)]
pub struct Variable {
    /// Name of the variable
    pub name: String,
    /// Value of the variable
    pub value: Expression,
}

impl Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = f.width().unwrap_or(0);
        write!(f, "{:indent$}", "\t")?;

        let name = &self.name;
        let value = &self.value;
        write!(f, "{name} = {value}")
    }
}

/// Reference to a variable
#[derive(Clone)]
pub struct VariableReference {
    /// Name of the variable
    pub name: String,
}

impl Display for VariableReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for VariableReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        write!(f, "{name}")
    }
}

/// Reference to a member of a variable
#[derive(Clone)]
pub struct MemberReference {
    /// Variable to reference
    pub variable: VariableReference,
    /// Member to reference
    pub member: String,
}

impl Display for MemberReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for MemberReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variable = &self.variable;
        let member = &self.member;
        write!(f, "{variable}.{member}")
    }
}

/// Call to a function or a call
#[derive(Clone)]
pub struct Call {
    /// Called entity
    pub callee: Callee,
    /// Arguments to pass to the function
    pub arguments: Vec<VariableReference>,
}

impl Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self.callee {
            Callee::Function(name) => name,
            Callee::Scope(scope) => &scope.name,
            Callee::RecursiveScope(name) => name,
        };
        let mut call = f.debug_tuple(name);
        for arg in self.arguments.iter() {
            call.field(arg);
        }
        call.finish()?;
        if let Callee::Scope(scope) = &self.callee {
            writeln!(f, "")?;

            let indent = f.width().unwrap_or(0);
            write!(f, "{scope:indent$}")?;
        }
        Ok(())
    }
}

/// Called entity
#[derive(Debug, Clone, From, TryInto)]
pub enum Callee {
    /// Call to function
    Function(String),
    /// Call to child scope
    #[from]
    Scope(Scope),
    /// Call to the parent scope
    RecursiveScope(String),
}

/// Switch between two values depending on a condition.
///
/// # Note
/// Values must have the same type
#[derive(Clone)]
pub struct Switch {
    /// Condition to switch
    pub condition: VariableReference,
    /// Expression to assign if condition is true
    pub on_true: Expression,
    /// Expression to assign if condition is false
    pub on_false: Expression,
}

impl Display for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Switch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let condition = &self.condition;
        let on_true = &self.on_true;
        let on_false = &self.on_false;
        if on_true.creates_scope() || on_false.creates_scope() {
            let indent = f.width().unwrap_or(0);
            let next_indent = indent + 1;

            writeln!(f, "if {condition}:")?;
            writeln!(f, "{on_true:next_indent$}")?;
            writeln!(f, "{:indent$}else:", "\t")?;
            writeln!(f, "{on_false:next_indent$}")?;
        } else {
            write!(f, "{on_true} if {condition} else {on_false}")?;
        }
        Ok(())
    }
}

/// Expression is a literal, variable reference, function call or scope call
#[derive(Clone, From, TryInto, Display, DebugCustom)]
pub enum Expression {
    /// Literal value
    Literal(hir::Literal),
    /// Reference to a member of a variable
    MemberReference(MemberReference),
    /// Reference to a variable
    VariableReference(VariableReference),
    /// Call to a function or a scope
    Call(Call),
    /// Switch between two values depending on a condition.
    ///
    /// # Note
    /// Values must have the same type
    Switch(Box<Switch>),
}

impl Expression {
    /// Check if this expression creates a scope
    pub fn creates_scope(&self) -> bool {
        match self {
            Expression::Call(call) => matches!(call.callee, Callee::Scope(_)),
            Expression::Switch(switch) => {
                switch.on_true.creates_scope() || switch.on_false.creates_scope()
            }
            _ => false,
        }
    }
}
