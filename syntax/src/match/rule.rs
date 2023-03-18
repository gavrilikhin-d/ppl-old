use std::{collections::HashMap, ops::Index};

use crate::{Match, PatternMatch};

/// Rule match info
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleMatch<'source> {
    /// Rule name
    pub rule: String,
    /// Matched patterns
    pub matched: Vec<PatternMatch<'source>>,
    /// Matched named captures
    pub named: HashMap<String, usize>,
}

impl<'source> RuleMatch<'source> {
    /// Get matched pattern by name
    pub fn get(&self, name: &str) -> Option<&PatternMatch<'source>> {
        self.named.get(name).map(|i| &self.matched[*i])
    }
}

impl<'source> Index<&str> for RuleMatch<'source> {
    type Output = PatternMatch<'source>;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'source> Match<'source> for RuleMatch<'source> {
    fn source(&self) -> &'source str {
        self.matched.first().unwrap().source()
    }

    fn range(&self) -> std::ops::Range<usize> {
        self.matched.first().unwrap().range()
    }
}
