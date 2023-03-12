use std::{
    collections::HashMap,
    ops::{Index, Range},
};

use regex::Regex;

use derive_more::From;

/// Syntax pattern
#[derive(Debug, From)]
pub enum Pattern {
    /// Match token using regex
    Regex(Regex),
    /// Reference to another rule
    Rule(String),
    /// Capture a pattern
    Capture {
        /// Name of the capture
        name: String,
        /// Pattern to capture
        pattern: Box<Pattern>,
    },
}

impl Pattern {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'t>(
        &self,
        source: &'t str,
        start: usize,
        parser: &Parser,
    ) -> Result<PatternMatch<'t>, ()> {
        match self {
            Pattern::Regex(regex) => {
                if let Some(m) = regex.find(&source[start..]) {
                    Ok(RegexMatch {
                        source,
                        start: start,
                        end: start + m.range().len(),
                    }
                    .into())
                } else {
                    Err(())
                }
            }
            Pattern::Rule(rule) => Ok(parser
                .rule(rule)
                .ok_or(())?
                .apply(source, start, parser)?
                .into()),
            Pattern::Capture { name, pattern } => {
                if let Ok(m) = pattern.apply(source, start, parser) {
                    Ok(PatternMatch::Capture {
                        name: name.clone(),
                        matched: Box::new(m),
                    })
                } else {
                    Err(())
                }
            }
        }
    }
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
                pos += m.range().len()
            }

            let m = pattern.apply(source, pos, parser)?;
            pos += m.range().len();
            if let PatternMatch::Capture { name, .. } = &m {
                named.insert(name.clone(), matches.len());
            }
            matches.push(m);
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
    Regex(RegexMatch<'t>),
    /// Matched with another rule
    Rule(RuleMatch<'t>),
    /// Captured pattern
    Capture {
        /// Name of the capture
        name: String,
        /// Matched pattern
        matched: Box<PatternMatch<'t>>,
    },
}

impl<'t> PatternMatch<'t> {
    /// Start position of the match in source in bytes
    pub fn start(&self) -> usize {
        match self {
            PatternMatch::Regex(m) => m.start(),
            PatternMatch::Rule(m) => m.start(),
            PatternMatch::Capture { matched, .. } => matched.start(),
        }
    }

    /// End position of the match in source in bytes
    pub fn end(&self) -> usize {
        match self {
            PatternMatch::Regex(m) => m.end(),
            PatternMatch::Rule(m) => m.end(),
            PatternMatch::Capture { matched, .. } => matched.end(),
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
            PatternMatch::Capture { matched, .. } => matched.as_str(),
        }
    }
}

/// Match represents a single match of a regex in a haystack.
///
/// The lifetime parameter `'t` refers to the lifetime of the matched text.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RegexMatch<'t> {
    pub(crate) source: &'t str,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl<'t> RegexMatch<'t> {
    /// Returns the starting byte offset of the match in the haystack.
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the ending byte offset of the match in the haystack.
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the range over the starting and ending byte offsets of the
    /// match in the haystack.
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// Returns the matched text.
    #[inline]
    pub fn as_str(&self) -> &'t str {
        &self.source[self.range()]
    }
}

impl<'t> From<RegexMatch<'t>> for &'t str {
    fn from(m: RegexMatch<'t>) -> &'t str {
        m.as_str()
    }
}

impl<'t> From<RegexMatch<'t>> for Range<usize> {
    fn from(m: RegexMatch<'t>) -> Range<usize> {
        m.range()
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
    /// Matched named captures
    named: HashMap<String, usize>,
}

// Implement rule["name"] syntax
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

    /// Get matched pattern by name
    pub fn get(&self, name: &str) -> Option<&PatternMatch<'t>> {
        self.named.get(name).map(|i| &self.matches[*i])
    }
}

impl<'t> Index<&str> for RuleMatch<'t> {
    type Output = PatternMatch<'t>;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(index).unwrap()
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

        let rule = rule.unwrap();
        let name = rule.get("name");
        assert_eq!(name.map(|m| m.as_str()), Some("Test"));

        let name = name.unwrap();
        assert_eq!(name.range(), 7..11);
    }
}
