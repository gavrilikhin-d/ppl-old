use std::ops::Range;

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
    fn source(&self) -> &'source str {
        self.matched.source()
    }

    fn range(&self) -> Range<usize> {
        self.matched.range()
    }
}
