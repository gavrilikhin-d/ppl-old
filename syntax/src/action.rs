use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Throw value as an error
    Throw(Value),
    /// Return value
    Return(Value),
}

impl Action {
    /// Get wrapped value that is either returned or thrown
    pub fn value(&self) -> &Value {
        match self {
            Action::Throw(value) => value,
            Action::Return(value) => value,
        }
    }
}

/// A JSON value that represents a variable reference.
pub fn reference(name: &str) -> serde_json::Value {
    json!({ "Variable": name })
}
