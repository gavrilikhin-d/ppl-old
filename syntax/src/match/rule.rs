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
    /// Check if match has no error nodes
    fn is_ok(&self) -> bool {
        self.matched.iter().all(|m| m.is_ok())
    }

    /// Get matched tokens
    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        Box::new(self.matched.iter().map(|m| m.tokens()).flatten())
    }

    fn submatches(&self) -> Box<dyn Iterator<Item = &PatternMatch<'source>> + '_> {
        Box::new(self.matched.iter())
    }
}
