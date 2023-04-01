use derive_more::From;
use regex::Regex;

mod group;
pub use group::*;

mod repeat;
pub use repeat::*;

use crate::{
    error::{UnexpectedEOF, UnexpectedToken},
    Parser, PatternMatch, SubsliceOffset,
};

/// Syntax pattern
#[derive(Debug, From)]
pub enum Pattern {
    /// Match token using regex
    Regex(Regex),
    /// Reference to another rule
    Rule(String),
    /// Group multiple patterns
    Group(Group),
    /// Repeat pattern
    Repeat(Repeat),
}

impl Pattern {
    /// Apply pattern to source, starting at `start` position
    pub fn apply<'source>(
        &self,
        source: &'source str,
        tokens: &mut (impl Iterator<Item = &'source str> + Clone),
        parser: &Parser,
    ) -> PatternMatch<'source> {
        match self {
            Pattern::Regex(regex) => {
                let token = tokens.next();
                if token.is_none() {
                    return UnexpectedEOF {
                        expected: regex.to_string(),
                        at: source.len(),
                    }
                    .into();
                }
                let token = token.unwrap();

                if regex.is_match(token) {
                    token.into()
                } else {
                    UnexpectedToken {
                        expected: regex.to_string(),
                        got: token.into(),
                        at: token
                            .offset_in(source)
                            .expect("Token isn't a subslice of source"),
                    }
                    .into()
                }
            }
            Pattern::Rule(rule) => {
                let rule = parser.try_rule(rule);
                if let Ok(rule) = rule {
                    rule.apply(source, tokens, parser).into()
                } else {
                    rule.err().unwrap().into()
                }
            }
            Pattern::Group(group) => group.apply(source, tokens, parser).into(),
            Pattern::Repeat(repeat) => repeat.apply(source, tokens, parser).into(),
        }
    }
}

impl TryFrom<&str> for Pattern {
    type Error = regex::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Pattern::Regex(Regex::new(&format!("^{value}$"))?))
    }
}
