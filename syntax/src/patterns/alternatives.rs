use crate::{Match, Parser, Pattern, PatternMatch};

/// Rule alternatives. Matches first pattern that matches.
pub struct Alternatives {
    /// Patterns to capture
    pub patterns: Vec<Pattern>,
}

impl Alternatives {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut (impl Iterator<Item = &'source str> + Clone),
        parser: &mut Parser,
    ) -> PatternMatch<'source> {
        for alt in &self.patterns {
            let tokens_copy = tokens.clone();
            let m = alt.apply(source, tokens, parser);
            if m.has_error() {
                *tokens = tokens_copy;
                continue;
            }

            return m;
        }

        unimplemented!("no alternatives matched")
    }
}
