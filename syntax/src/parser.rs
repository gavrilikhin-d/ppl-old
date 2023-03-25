use std::collections::HashMap;

use crate::{error::UnknownRule, Error, Pattern, Rule, RuleMatch};

/// Syntax parser
#[derive(Debug)]
pub struct Parser {
    /// Rule to start from
    root: String,
    /// Rules for the parser
    rules: Vec<Rule>,
    /// Mapping of rule names to indices
    rules_mapping: HashMap<String, usize>,
}

impl Parser {
    /// Add a rule to the parser
    pub fn add_rule(&mut self, rule: Rule) -> Result<(), ()> {
        if self.rules_mapping.contains_key(rule.name()) {
            return Err(());
        }
        let index = self.rules.len();
        self.rules_mapping.insert(rule.name().into(), index);
        self.rules.push(rule);
        Ok(())
    }

    /// Get a rule by name or return None
    pub fn rule(&self, name: &str) -> Option<&Rule> {
        let index = self.rules_mapping.get(name)?;
        let rule = &self.rules[*index];
        debug_assert_eq!(rule.name(), name);
        Some(rule)
    }

    /// Get a rule by name, or return an error
    pub fn try_rule(&self, name: &str) -> Result<&Rule, UnknownRule> {
        self.rule(name)
            .ok_or_else(|| UnknownRule { name: name.into() })
    }

    /// Parse a list of tokens, starting from the root rule.
    ///
    /// Tokens must be subslices of `source`.
    pub fn parse<'source>(
        &self,
        source: &'source str,
        mut token: impl Iterator<Item = &'source str>,
    ) -> Result<RuleMatch<'source>, Error> {
        self.try_rule(&self.root)?.apply(source, &mut token, self)
    }
}

impl Default for Parser {
    fn default() -> Self {
        let mut parser = Parser {
            root: "Syntax".into(),
            rules: Vec::new(),
            rules_mapping: HashMap::new(),
        };
        parser
            .add_rule(Rule {
                name: "Syntax".into(),
                patterns: vec![
                    "syntax".try_into().unwrap(),
                    Pattern::Capture {
                        name: "name".into(),
                        pattern: Box::new(Pattern::Rule("Identifier".into())),
                    },
                ],
            })
            .unwrap();
        parser
            .add_rule(Rule {
                name: "Identifier".into(),
                patterns: vec![r"[a-zA-Z_][a-zA-Z0-9_]*".try_into().unwrap()],
            })
            .unwrap();
        parser
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_rule() {
        let parser = Parser::default();
        let rule = parser.try_rule("Unknown");
        assert_eq!(
            rule.err(),
            Some(UnknownRule {
                name: "Unknown".into()
            })
        );
    }

    #[test]
    fn rule() {
        let parser = Parser::default();

        let source = "syntax Test";
        let tokens = source.split_whitespace();
        let rule = parser.parse(source, tokens);
        assert!(rule.is_ok());

        let rule = rule.unwrap();
        let name = rule.get("name");
        assert_eq!(name.map(|m| m.tokens().next()).flatten(), Some("Test"));
    }
}
