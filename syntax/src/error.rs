#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum Error {
    /// Unknown rule
    #[error(transparent)]
    UnknownRule(#[from] UnknownRule),
    /// Uexpected token
    #[error(transparent)]
    UnexpectedToken(#[from] UnexpectedToken),
}

/// Error for unknown rule
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unknown rule '{name}'")]
pub struct UnknownRule {
    /// Rule's name
    pub name: String,
}

/// Error for unexpected token
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unexpected token '{got}', expected '{expected}'")]
pub struct UnexpectedToken {
    /// Expected token
    pub expected: String,
    /// Got token
    pub got: String,
}
