use std::collections::HashMap;

use crate::{Match, Parser, Pattern, PatternMatch, RuleMatch};

/// Syntax rule
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Patterns of the rule
    pub patterns: Vec<Pattern>,
    /// Action to perform when rule is matched
    pub action: Option<Box<dyn Fn(&mut Parser, &mut RuleMatch<'_>) -> ()>>,
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
        token: &mut (impl Iterator<Item = &'source str> + Clone),
        parser: &mut Parser,
    ) -> RuleMatch<'source> {
        let mut matched = Vec::new();
        let mut named = HashMap::new();

        for pattern in &self.patterns {
            let m = pattern.apply(source, token, parser);

            if let PatternMatch::Group(g) = &m {
                if !g.name.is_empty() {
                    named.insert(g.name.clone(), matched.len());
                }
            }
            matched.push(m);
        }

        let mut m = RuleMatch {
            rule: self.name.clone(),
            matched,
            named,
            payload: None,
        };
        if m.is_ok() {
            if let Some(action) = &self.action {
                action(parser, &mut m);
            }
        }
        m
    }
}
