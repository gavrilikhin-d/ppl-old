#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum Error {
    /// Unexpected token
    #[error(transparent)]
    UnexpectedToken(#[from] UnexpectedToken),
    /// Unexpected end of input
    #[error(transparent)]
    UnexpectedEOF(#[from] UnexpectedEOF),
}

/// Error for unexpected token
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unexpected token '{got}', expected '{expected}'")]
pub struct UnexpectedToken {
    /// Expected token
    pub expected: String,
    /// Got token
    pub got: String,
    /// Start position of the `got` token
    pub at: usize,
}

/// Error for unexpected end of input
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[error("Unexpected end of input, expected '{expected}'")]
pub struct UnexpectedEOF {
    /// Expected token
    pub expected: String,
    /// Position where token was expected
    pub at: usize,
}
