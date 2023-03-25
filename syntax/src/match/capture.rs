use crate::{Match, PatternMatch};

/// Captured pattern
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CaptureMatch<'source> {
    /// Name of the capture
    pub name: String,
    /// Matched pattern
    pub matched: Box<PatternMatch<'source>>,
}

impl<'source> Match<'source> for CaptureMatch<'source> {
    fn is_ok(&self) -> bool {
        self.matched.is_ok()
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        self.matched.tokens()
    }
}
