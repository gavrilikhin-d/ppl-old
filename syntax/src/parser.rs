use std::collections::HashMap;

use crate::{error::UnknownRule, Error, MatchError, Pattern, Rule, RuleMatch};

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

    /// Parse a string, starting from root rule
    pub fn parse<'source>(
        &self,
        source: &'source str,
    ) -> Result<RuleMatch<'source>, MatchError<'source>> {
        self.try_rule(&self.root)
            .map_err(|e| MatchError {
                source,
                start: 0,
                end: 0,
                payload: e.into(),
            })?
            .apply(source, 0, self)
    }
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            root: "Syntax".into(),
            rules: vec![
                Rule {
                    name: "Syntax".into(),
                    patterns: vec![
                        r"^syntax".try_into().unwrap(),
                        Pattern::Capture {
                            name: "name".into(),
                            pattern: Box::new(Pattern::Rule("Identifier".into())),
                        },
                    ],
                },
                Rule {
                    name: "Identifier".into(),
                    patterns: vec![r"^[a-zA-Z_][a-zA-Z0-9_]*".try_into().unwrap()],
                },
            ],
            rules_mapping: vec![("Syntax".into(), 0), ("Identifier".into(), 1)]
                .into_iter()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Match;

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

        let rule = parser.parse("syntax Test");
        assert!(rule.is_ok());

        let rule = rule.unwrap();
        let name = rule.get("name");
        assert_eq!(name.map(|m| m.as_str()), Some("Test"));

        let name = name.unwrap();
        assert_eq!(name.range(), 7..11);
    }
}
