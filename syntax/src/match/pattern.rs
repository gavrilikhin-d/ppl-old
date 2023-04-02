use derive_more::From;

use crate::{Error, GroupMatch, Match, RepeatMatch, RuleMatch};

/// Pattern match info
#[derive(From)]
pub enum PatternMatch<'source> {
    /// Matched with regex
    #[from]
    Regex(&'source str),
    /// Matched with another rule
    #[from]
    Rule(RuleMatch<'source>),
    /// Matched group
    #[from]
    Group(GroupMatch<'source>),
    /// Matched repeated patterns
    #[from]
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

    fn submatches(&self) -> Box<dyn Iterator<Item = &PatternMatch<'source>> + '_> {
        match self {
            PatternMatch::Regex(_) => Box::new(std::iter::empty()),
            PatternMatch::Rule(r) => r.submatches(),
            PatternMatch::Group(g) => g.submatches(),
            PatternMatch::Repeat(r) => r.submatches(),
            PatternMatch::Error(_) => Box::new(std::iter::empty()),
        }
    }
}

impl<T: Into<Error>> From<T> for PatternMatch<'_> {
    fn from(e: T) -> Self {
        Self::Error(e.into())
    }
}
