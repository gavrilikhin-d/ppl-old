use std::collections::HashMap;

use regex::Regex;

use crate::{Match, MatchError, Parser, Pattern, PatternMatch, RuleMatch};

/// Syntax rule
#[derive(Debug)]
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Patterns of the rule
    pub patterns: Vec<Pattern>,
}

impl Rule {
    /// Get name of the rule
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Apply rule to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        start: usize,
        parser: &Parser,
    ) -> Result<RuleMatch<'source>, MatchError<'source>> {
        assert!(!self.patterns.is_empty());

        let mut pos = start;
        let mut matched = Vec::new();
        let mut named = HashMap::new();

        let ws = Regex::new(r"^\s+").unwrap();

        for pattern in &self.patterns {
            if let Some(m) = ws.find(&source[pos..]) {
                pos += m.range().len()
            }

            let m = pattern.apply(source, pos, parser)?;
            pos += m.range().len();
            if let PatternMatch::Capture(c) = &m {
                named.insert(c.name.clone(), matched.len());
            }
            matched.push(m);
        }

        Ok(RuleMatch {
            rule: self.name.clone(),
            matched,
            named,
        })
    }
}
