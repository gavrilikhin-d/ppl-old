use std::ops::Range;

mod capture;
pub use capture::*;

mod pattern;
pub use pattern::*;

mod regex;
pub use crate::r#match::regex::*;

mod rule;
pub use rule::*;

/// Result of applying pattern or rule
pub trait Match<'source> {
    /// Source string
    fn source(&self) -> &'source str;

    /// Start position of match
    fn start(&self) -> usize {
        self.range().start
    }

    /// End position of match (exclusive)
    fn end(&self) -> usize {
        self.range().end
    }

    /// Range of the match in source in bytes
    fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    /// Matched text
    fn as_str(&self) -> &'source str {
        &self.source()[self.range()]
    }
}

// Macro that generates From<$M> for &'source str and Range<usize>
macro_rules! impl_from_match {
    ($M: ident) => {
        impl<'source> From<$M<'source>> for &'source str {
            fn from(m: $M<'source>) -> &'source str {
                m.as_str()
            }
        }

        impl<'source> From<$M<'source>> for Range<usize> {
            fn from(m: $M<'source>) -> Range<usize> {
                m.range()
            }
        }
    };
}

impl_from_match!(RegexMatch);
impl_from_match!(RuleMatch);
impl_from_match!(CaptureMatch);
impl_from_match!(PatternMatch);
