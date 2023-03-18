use crate::Match;

/// Match represents a single match of a regex in a source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexMatch<'source> {
    /// Source text
    pub source: &'source str,
    /// Start position of the match in source in bytes
    pub start: usize,
    /// End position of the match in source in bytes (exclusive)
    pub end: usize,
}

impl<'source> Match<'source> for RegexMatch<'source> {
    fn source(&self) -> &'source str {
        self.source
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
