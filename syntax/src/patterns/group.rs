use crate::{GroupMatch, Pattern};

/// Group multiple patterns
pub struct Group {
    /// Name of the group
    pub name: String,
    /// Patterns to capture
    pub patterns: Vec<Pattern>,
}

impl Group {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut (impl Iterator<Item = &'source str> + Clone),
    ) -> GroupMatch<'source> {
        let mut matched = Vec::new();

        for pattern in &self.patterns {
            matched.push(pattern.apply(source, tokens));
        }

        GroupMatch {
            name: self.name.clone(),
            matched,
        }
    }
}
