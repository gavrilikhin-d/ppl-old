use std::{collections::HashMap, ops::Range};

use regex::Regex;

use derive_more::From;

/// Syntax pattern
#[derive(Debug, From)]
pub enum Pattern {
    /// Match token using regex
    Regex(Regex),
    /// Reference to another rule
    Rule(String),
}

/// Syntax rule
#[derive(Debug)]
pub struct Rule {
    /// Rule name
    name: String,
    /// Patterns of the rule
    patterns: Vec<Pattern>,
}

impl Rule {
    /// Apply rule to source, starting at `start` position
    pub fn apply<'t>(
        &self,
        source: &'t str,
        start: usize,
        parser: &Parser,
    ) -> Result<RuleMatch<'t>, ()> {
        let mut pos = start;
        let mut matches = Vec::new();
        let mut named = HashMap::new();

        let ws = Regex::new(r"^\s+").unwrap();

        for pattern in &self.patterns {
            if let Some(m) = ws.find(&source[pos..]) {
                pos += m.end()
            }

            match pattern {
                Pattern::Regex(regex) => {
                    if let Some(m) = regex.find(&source[pos..]) {
                        pos += m.end();
                        matches.push(PatternMatch::Regex(m));
                    } else {
                        return Err(());
                    }
                }
                Pattern::Rule(rule) => {
                    if let Ok(m) = parser.rule(rule).ok_or(())?.apply(source, pos, parser) {
                        pos = m.end();
                        matches.push(PatternMatch::Rule(m));
                    } else {
                        return Err(());
                    }
                }
            }
        }

        Ok(RuleMatch {
            rule: self.name.clone(),
            source,
            matches,
            named,
        })
    }
}

/// Pattern match info
#[derive(Debug, Clone, Eq, PartialEq, From)]
pub enum PatternMatch<'t> {
    /// Matched with regex
    Regex(regex::Match<'t>),
    /// Matched with another rule
    Rule(RuleMatch<'t>),
}

impl<'t> PatternMatch<'t> {
    /// Start position of the match in source in bytes
    pub fn start(&self) -> usize {
        match self {
            PatternMatch::Regex(m) => m.start(),
            PatternMatch::Rule(m) => m.start(),
        }
    }

    /// End position of the match in source in bytes
    pub fn end(&self) -> usize {
        match self {
            PatternMatch::Regex(m) => m.end(),
            PatternMatch::Rule(m) => m.end(),
        }
    }

    /// Range of the match in source in bytes
    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    /// Matched text
    pub fn as_str(&self) -> &'t str {
        match self {
            PatternMatch::Regex(m) => m.as_str(),
            PatternMatch::Rule(m) => m.as_str(),
        }
    }
}

/// Rule match info
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleMatch<'t> {
    /// Rule name
    rule: String,
    /// Source text
    source: &'t str,
    /// Matched patterns
    matches: Vec<PatternMatch<'t>>,
    /// Named match patterns patterns
    named: HashMap<String, PatternMatch<'t>>,
}

impl<'t> RuleMatch<'t> {
    /// Start position of the match in source in bytes
    pub fn start(&self) -> usize {
        self.matches.first().unwrap().start()
    }

    /// End position of the match in source in bytes
    pub fn end(&self) -> usize {
        self.matches.last().unwrap().end()
    }

    /// Range of the match in source in bytes
    pub fn range(&self) -> Range<usize> {
        self.start()..self.end()
    }

    /// Matched text
    pub fn as_str(&self) -> &'t str {
        &self.source[self.range()]
    }
}

impl TryFrom<&str> for Pattern {
    type Error = regex::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Pattern::Regex(Regex::new(value)?))
    }
}

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

impl Default for Parser {
    fn default() -> Self {
        Parser {
            root: "Syntax".into(),
            rules: vec![
                Rule {
                    name: "Syntax".into(),
                    patterns: vec![
                        r"^syntax".try_into().unwrap(),
                        Pattern::Rule("Identifier".into()),
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

impl Parser {
    /// Get a rule by name
    pub fn rule(&self, name: &str) -> Option<&Rule> {
        let index = self.rules_mapping.get(name)?;
        let rule = &self.rules[*index];
        debug_assert_eq!(rule.name, name);
        Some(rule)
    }

    /// Parse a string, starting from root rule
    pub fn parse<'t>(&self, source: &'t str) -> Result<RuleMatch<'t>, ()> {
        self.rule(&self.root).ok_or(())?.apply(source, 0, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule() {
        let parser = Parser::default();

        let rule = parser.parse("syntax Test");
        assert!(rule.is_ok());
    }
}
