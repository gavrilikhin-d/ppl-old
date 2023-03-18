#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum Error {
    /// Unknown rule
    #[error(transparent)]
    UnknownRule(#[from] UnknownRule),
}

/// Error for unknown rule
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unknown rule '{name}'")]
pub struct UnknownRule {
    /// Rule's name
    pub name: String,
}
