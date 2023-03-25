use crate::PatternMatch;

/// Matched group
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GroupMatch<'source> {
    /// Matched patterns
    pub matched: Vec<PatternMatch<'source>>,
}

impl<'source> GroupMatch<'source> {
    /// Get matched tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        Box::new(self.matched.iter().map(|m| m.tokens()).flatten())
    }
}
