use serde_json::json;

/// A JSON value that represents a variable reference.
pub fn reference(name: &str) -> serde_json::Value {
    json!({ "Variable": name })
}
