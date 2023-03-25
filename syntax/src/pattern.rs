use derive_more::From;
use regex::Regex;

use crate::{
    error::UnexpectedToken, CaptureMatch, Error, GroupMatch, Parser, PatternMatch, SubsliceOffset,
};

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
    /// Match a group of patterns
    Group {
        /// Patterns to match
        patterns: Vec<Pattern>,
    },
}

impl Pattern {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut impl Iterator<Item = &'source str>,
        parser: &Parser,
    ) -> PatternMatch<'source> {
        match self {
            Pattern::Regex(regex) => {
                let token = tokens.next();
                if token.is_none() {
                    unimplemented!("error")
                }
                let token = token.unwrap();

                if regex.is_match(token) {
                    token.into()
                } else {
                    Error::from(UnexpectedToken {
                        expected: regex.to_string(),
                        got: token.into(),
                        at: token
                            .offset_in(source)
                            .expect("Token isn't a subslice of source"),
                    })
                    .into()
                }
            }
            Pattern::Rule(rule) => {
                let rule = parser.try_rule(rule);
                if let Ok(rule) = rule {
                    rule.apply(source, tokens, parser).into()
                } else {
                    Error::from(rule.err().unwrap()).into()
                }
            }
            Pattern::Capture { name, pattern } => {
                let m = pattern.apply(source, tokens, parser);
                CaptureMatch {
                    name: name.clone(),
                    matched: Box::new(m),
                }
                .into()
            }
            Pattern::Group { patterns } => {
                let mut matched = Vec::new();

                for pattern in patterns {
                    let m = pattern.apply(source, tokens, parser);
                    matched.push(m);
                }

                GroupMatch { matched }.into()
            }
        }
    }
}

impl TryFrom<&str> for Pattern {
    type Error = regex::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Pattern::Regex(Regex::new(&format!("^{value}$"))?))
    }
}
