mod repeat;

use derive_more::From;
pub use repeat::*;

/// Possible patterns
#[derive(Debug, PartialEq, Clone, From)]
pub enum Pattern {
    /// Reference to another rule
    RuleReference(String),
    /// Group of patterns
    Group(Vec<Pattern>),
    /// Regex
    Regex(String),
    /// Pattern alternatives
    Alternatives(Vec<Pattern>),
    /// Repeat pattern
    #[from]
    Repeat(Repeat),
}
