use crate::PatternMatch;

/// Captured pattern
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CaptureMatch<'source> {
    /// Name of the capture
    pub name: String,
    /// Matched pattern
    pub matched: Box<PatternMatch<'source>>,
}

impl<'source> CaptureMatch<'source> {
    /// Get matched tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        self.matched.tokens()
    }
}
