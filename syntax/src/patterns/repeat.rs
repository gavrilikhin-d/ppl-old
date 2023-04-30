use crate::parsers::{ParseResult, Parser};

use super::Pattern;

/// Repeat pattern
#[derive(Debug, PartialEq, Clone)]
pub struct Repeat {
    /// Pattern to repeat
    pub pattern: Box<Pattern>,
    /// Minimum number of repetitions
    pub at_least: usize,
    /// Maximum number of repetitions
    pub at_most: Option<usize>,
}

impl Repeat {
    /// Repeat pattern zero or more times (x*)
    pub fn zero_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: None,
        }
    }

    /// Repeat pattern once or more times (x+)
    pub fn once_or_more(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 1,
            at_most: None,
        }
    }

    /// Repeat pattern at most once (x?)
    pub fn at_most_once(pattern: Pattern) -> Self {
        Self {
            pattern: Box::new(pattern),
            at_least: 0,
            at_most: Some(1),
        }
    }
}
