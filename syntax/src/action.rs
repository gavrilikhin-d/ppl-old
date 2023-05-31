use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::errors::Error;

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

    /// Execute this action with expanding variables
    pub fn execute(&self, variables: &Map<String, Value>) -> Result<Value, Value> {
        let ast = expand_variables(self.value(), variables);
        if matches!(self, Action::Throw(_)) {
            let error: Error = serde_json::from_value(ast.clone()).unwrap();
            println!("{:?}", miette::Report::new(error));
            return Err(ast);
        }
        Ok(ast)
    }
}

/// A JSON value that represents a variable reference.
pub fn reference(name: &str) -> Value {
    json!({ "Variable": name })
}

/// Replace `{ "Variable": name }` with value of variable
pub fn expand_variables(action: &Value, ast: &Map<String, Value>) -> Value {
    match action {
        serde_json::Value::Object(o) => {
            if o.keys().len() == 1 && o.keys().next().unwrap() == "Variable" {
                let variable = o.get("Variable").unwrap().as_str().unwrap();
                return ast.get(variable).unwrap().clone();
            }

            let mut result = serde_json::Map::new();
            for (key, value) in o {
                result.insert(key.clone(), expand_variables(value, ast));
            }
            result.into()
        }
        serde_json::Value::Array(a) => {
            let mut result = Vec::new();
            for value in a {
                result.push(expand_variables(value, ast));
            }
            result.into()
        }
        _ => action.clone(),
    }
}
