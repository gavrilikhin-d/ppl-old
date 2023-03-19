use std::fmt::Display;

use crate::Error;

use crate::Match;

/// Error node
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub struct MatchError<'source> {
    /// Source string
    pub source: &'source str,
    /// Error's start position
    pub start: usize,
    /// Error's end position (exclusive)
    pub end: usize,
    /// Error payload
    #[source]
    pub payload: Error,
}

impl<'source> Match<'source> for MatchError<'source> {
    fn source(&self) -> &'source str {
        self.source
    }

    fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

impl Display for MatchError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.payload)
    }
}
