use derive_more::From;

use crate::{Error, GroupMatch, Match, RepeatMatch, RuleMatch};

/// Pattern match info
#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum PatternMatch<'source> {
    /// Matched with regex
    Regex(&'source str),
    /// Matched with another rule
    Rule(RuleMatch<'source>),
    /// Matched group
    Group(GroupMatch<'source>),
    /// Matched repeated patterns
    Repeat(RepeatMatch<'source>),
    /// Error node
    Error(Error),
}

impl<'source> Match<'source> for PatternMatch<'source> {
    fn is_ok(&self) -> bool {
        match self {
            PatternMatch::Regex(_) => true,
            PatternMatch::Rule(r) => r.is_ok(),
            PatternMatch::Group(g) => g.is_ok(),
            PatternMatch::Repeat(r) => r.is_ok(),
            PatternMatch::Error(_) => false,
        }
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        match self {
            PatternMatch::Regex(s) => Box::new(std::iter::once(*s)),
            PatternMatch::Rule(r) => r.tokens(),
            PatternMatch::Group(g) => g.tokens(),
            PatternMatch::Repeat(r) => r.tokens(),
            PatternMatch::Error(_) => Box::new(std::iter::empty()),
        }
    }
}
