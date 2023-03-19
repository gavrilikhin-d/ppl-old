use derive_more::From;

use crate::{CaptureMatch, Match, MatchError, RegexMatch, RuleMatch};

/// Pattern match info
#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum PatternMatch<'source> {
    /// Matched with regex
    Regex(RegexMatch<'source>),
    /// Matched with another rule
    Rule(RuleMatch<'source>),
    /// Captured pattern
    Capture(CaptureMatch<'source>),
    /// Error node
    Error(MatchError<'source>),
}

impl<'source> Match<'source> for PatternMatch<'source> {
    fn source(&self) -> &'source str {
        match self {
            PatternMatch::Regex(m) => m.source(),
            PatternMatch::Rule(m) => m.source(),
            PatternMatch::Capture(m) => m.source(),
            PatternMatch::Error(m) => m.source(),
        }
    }

    fn range(&self) -> std::ops::Range<usize> {
        match self {
            PatternMatch::Regex(m) => m.range(),
            PatternMatch::Rule(m) => m.range(),
            PatternMatch::Capture(m) => m.range(),
            PatternMatch::Error(m) => m.range(),
        }
    }
}
