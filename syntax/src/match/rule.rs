use std::{any::Any, collections::HashMap, error::Error, ops::Index};

use crate::{Match, PatternMatch};

/// Rule match info
pub struct RuleMatch<'source> {
    /// Rule name
    pub rule: String,
    /// Matched patterns
    pub matched: Vec<PatternMatch<'source>>,
    /// Matched named captures
    pub named: HashMap<String, usize>,
    /// Result of running action, if any
    pub action_result: Option<Result<Box<dyn Any>, Box<dyn Error>>>,
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
    fn has_error(&self) -> bool {
        self.action_result.as_ref().is_some_and(|r| r.is_err())
            || self.matched.iter().any(|m| m.has_error())
    }

    fn tokens(&self) -> Box<dyn Iterator<Item = &'source str> + '_> {
        Box::new(self.matched.iter().map(|m| m.tokens()).flatten())
    }

    fn submatches(&self) -> Box<dyn Iterator<Item = &PatternMatch<'source>> + '_> {
        Box::new(self.matched.iter())
    }
}
