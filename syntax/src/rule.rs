use std::collections::HashMap;

use crate::{Error, Parser, Pattern, PatternMatch, RuleMatch};

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
        token: &mut impl Iterator<Item = &'source str>,
        parser: &Parser,
    ) -> Result<RuleMatch<'source>, Error> {
        let mut matched = Vec::new();
        let mut named = HashMap::new();

        for pattern in &self.patterns {
            let m = pattern.apply(source, token, parser)?;

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
