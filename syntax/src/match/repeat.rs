use crate::{Match, PatternMatch};

/// Matched group
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RepeatMatch<'source> {
    /// Minimum expected number of matches
    pub at_least: usize,
    /// Maximum expected number of matches
    pub at_most: Option<usize>,
    /// Matched patterns
    pub matched: Vec<PatternMatch<'source>>,
}

impl<'source> Match<'source> for RepeatMatch<'source> {
    fn is_ok(&self) -> bool {
        self.matched.iter().all(|m| m.is_ok())
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        Box::new(self.matched.iter().map(|m| m.tokens()).flatten())
    }

    fn submatches(&self) -> Box<dyn Iterator<Item = &PatternMatch<'source>> + '_> {
        Box::new(self.matched.iter())
    }
}
