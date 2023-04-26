/// Creates recoverable error for nom
#[macro_export]
macro_rules! err {
    ($error: expr) => {
        Err(nom::Err::Error(Box::new($error)))
    };
}

#[derive(Debug, thiserror::Error)]
#[error("Regex didn't match")]
pub struct RegexMismatch {}

#[derive(Debug, thiserror::Error)]
#[error("Unknown rule reference")]
pub struct UnknownRuleReference {}
