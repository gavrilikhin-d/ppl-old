use derive_more::From;

use crate::{CaptureMatch, Error, GroupMatch, RuleMatch};

/// Pattern match info
#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum PatternMatch<'source> {
    /// Matched with regex
    Regex(&'source str),
    /// Matched with another rule
    Rule(RuleMatch<'source>),
    /// Captured pattern
    Capture(CaptureMatch<'source>),
    /// Matched group
    Group(GroupMatch<'source>),
    /// Error node
    Error(Error),
}

impl<'source> PatternMatch<'source> {
    /// Get matched tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        match self {
            PatternMatch::Regex(s) => Box::new(std::iter::once(*s)),
            PatternMatch::Rule(r) => r.tokens(),
            PatternMatch::Capture(c) => c.tokens(),
            PatternMatch::Group(g) => g.tokens(),
            PatternMatch::Error(_) => Box::new(std::iter::empty()),
        }
    }
}
